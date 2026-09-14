/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::path::{Path, PathBuf};

use base64::Engine;
use hex::FromHex;
use ssh2;

use crate::file_handler;
use crate::secrets_manager::{self, SecretStore, KEYRING_PREFIX};
use crate::utils::sha256;

/// Auth material for a remote core host SSH session (parity with the ssh connector).
#[derive(Clone, Default)]
pub struct SshAuthOptions {
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub private_key_passphrase: Option<String>,
    pub agent_key_identifier: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostKeyChallenge {
    pub message: String,
    pub key_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SshAuthError {
    HostKeyUnverified(HostKeyChallenge),
    Other(String),
}

impl SshAuthError {
    pub fn other(message: impl Into<String>) -> Self {
        SshAuthError::Other(message.into())
    }

    pub fn to_ui_message(&self) -> String {
        match self {
            SshAuthError::HostKeyUnverified(challenge) => {
                format!("{}\n\n{}", challenge.message, challenge.key_id)
            }
            SshAuthError::Other(message) => message.clone(),
        }
    }
}

impl From<SshAuthError> for String {
    fn from(error: SshAuthError) -> Self {
        error.to_ui_message()
    }
}

pub fn known_hosts_path(custom_path: Option<&str>) -> Result<PathBuf, SshAuthError> {
    if let Some(custom) = custom_path.filter(|path| !path.is_empty()) {
        let path = PathBuf::from(custom);
        if !path.exists() {
            return Err(SshAuthError::other(format!(
                "No such file for custom_known_hosts_path: {}",
                path.display()
            )));
        }
        return Ok(path);
    }

    let path = file_handler::get_config_dir().join("known_hosts");
    if !path.exists() {
        std::fs::File::create(&path).map_err(|error| {
            SshAuthError::other(format!("Failed to create known_hosts {}: {}", path.display(), error))
        })?;
    }
    Ok(path)
}

pub fn host_key_id(key_type: ssh2::HostKeyType, key: &[u8]) -> Result<String, SshAuthError> {
    let fp_hex = sha256::hash(key);
    let fp_bytes = Vec::<u8>::from_hex(fp_hex.clone())
        .map_err(|error| SshAuthError::other(error.to_string()))?;
    let fp_base64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(fp_bytes);

    Ok(format!(
        "{:?} {}\n\nFingerprints:\nSHA256 (hex): {}\nSHA256 (base64): {}",
        key_type,
        base64::engine::general_purpose::STANDARD_NO_PAD.encode(key),
        fp_hex,
        fp_base64
    ))
}

pub fn check_known_hosts(
    session: &ssh2::Session,
    hostname: &str,
    port: u16,
    custom_known_hosts_path: Option<&str>,
) -> Result<(), SshAuthError> {
    let known_hosts_path = known_hosts_path(custom_known_hosts_path)?;
    let mut known_hosts = session
        .known_hosts()
        .map_err(|error| SshAuthError::other(format!("Failed to initialize known_hosts: {}", error)))?;
    known_hosts
        .read_file(&known_hosts_path, ssh2::KnownHostFileKind::OpenSSH)
        .map_err(|error| SshAuthError::other(format!("Failed to read known_hosts: {}", error)))?;

    let (key, key_type) = session
        .host_key()
        .ok_or_else(|| SshAuthError::other("SSH server did not present a host key"))?;
    let key_id = host_key_id(key_type, key)?;

    match known_hosts.check_port(hostname, port, key) {
        ssh2::CheckResult::Match => Ok(()),
        ssh2::CheckResult::NotFound => Err(SshAuthError::HostKeyUnverified(HostKeyChallenge {
            message: format!(
                "Host key for '{}' was not found in known hosts.\nDo you want to trust this key:",
                hostname
            ),
            key_id,
        })),
        ssh2::CheckResult::Mismatch => Err(SshAuthError::HostKeyUnverified(HostKeyChallenge {
            message: format!(
                "Host key for '{}' HAS CHANGED! Do you trust this NEW key:",
                hostname
            ),
            key_id,
        })),
        ssh2::CheckResult::Failure => Err(SshAuthError::other(format!(
            "Failed to check host key for '{}'",
            hostname
        ))),
    }
}

/// Writes the current session host key to known_hosts after the user accepts `expected_key_id`.
pub fn accept_host_key(
    session: &ssh2::Session,
    hostname: &str,
    port: u16,
    expected_key_id: &str,
    custom_known_hosts_path: Option<&str>,
) -> Result<(), SshAuthError> {
    let known_hosts_path = known_hosts_path(custom_known_hosts_path)?;
    let mut known_hosts = session
        .known_hosts()
        .map_err(|error| SshAuthError::other(format!("Failed to initialize known_hosts: {}", error)))?;
    known_hosts
        .read_file(&known_hosts_path, ssh2::KnownHostFileKind::OpenSSH)
        .map_err(|error| SshAuthError::other(format!("Failed to read known_hosts: {}", error)))?;

    let (key, key_type) = session
        .host_key()
        .ok_or_else(|| SshAuthError::other("SSH server did not present a host key"))?;
    let key_id = host_key_id(key_type, key)?;
    if key_id != expected_key_id {
        return Err(SshAuthError::other("Host key changed again"));
    }

    let known_hosts_name = if port == 22 {
        hostname.to_string()
    }
    else {
        format!("[{}]:{}", hostname, port)
    };

    known_hosts
        .add(&known_hosts_name, key, hostname, key_type.into())
        .map_err(|error| SshAuthError::other(format!("Failed to add host key to known hosts: {}", error)))?;
    known_hosts
        .write_file(&known_hosts_path, ssh2::KnownHostFileKind::OpenSSH)
        .map_err(|error| SshAuthError::other(format!("Failed to write known hosts file: {}", error)))?;
    Ok(())
}

/// Authenticates like the ssh connector: password, else private key, else agent.
pub fn authenticate(session: &ssh2::Session, auth: &SshAuthOptions) -> Result<(), SshAuthError> {
    if let Some(password) = auth.password.as_ref().filter(|value| !value.is_empty()) {
        session
            .userauth_password(&auth.username, password)
            .map_err(|error| SshAuthError::other(format!("Failed to authenticate with password: {}", error)))?;
        return Ok(());
    }

    if let Some(private_key_path) = auth.private_key_path.as_ref().filter(|value| !value.is_empty()) {
        let path = Path::new(private_key_path);
        let passphrase = auth
            .private_key_passphrase
            .as_ref()
            .filter(|value| !value.is_empty())
            .map(|value| value.as_str());
        session
            .userauth_pubkey_file(&auth.username, None, path, passphrase)
            .map_err(|error| SshAuthError::other(format!("Failed to authenticate with private key: {}", error)))?;
        return Ok(());
    }

    let mut agent = session
        .agent()
        .map_err(|error| SshAuthError::other(format!("Failed to open SSH agent: {}", error)))?;
    agent
        .connect()
        .map_err(|error| SshAuthError::other(format!("Failed to connect to SSH agent: {}", error)))?;
    agent
        .list_identities()
        .map_err(|error| SshAuthError::other(format!("Failed to list SSH agent identities: {}", error)))?;

    let mut identities = agent
        .identities()
        .map_err(|error| SshAuthError::other(error.to_string()))?;
    if let Some(selected_id) = auth.agent_key_identifier.as_ref().filter(|value| !value.is_empty()) {
        identities.retain(|identity| identity.comment() == selected_id.as_str());
    }

    for identity in identities.iter() {
        ::log::debug!("Trying SSH auth with agent key \"{}\"", identity.comment());
        if agent.userauth(&auth.username, identity).is_ok() && session.authenticated() {
            return Ok(());
        }
    }

    Err(SshAuthError::other("Failed to authenticate with SSH agent"))
}

/// Resolves a profile secret field that may be plaintext or a `keyring:` placeholder.
pub fn resolve_secret_value(store: &dyn SecretStore, stored: &str) -> Result<Option<String>, SshAuthError> {
    if stored.is_empty() {
        return Ok(None);
    }
    if let Some(lookup_key) = stored.strip_prefix(KEYRING_PREFIX) {
        if lookup_key.is_empty() || lookup_key == "error" {
            return Err(SshAuthError::other("Core SSH secret placeholder is invalid"));
        }
        return store
            .get(lookup_key)
            .map_err(|error| SshAuthError::other(error.to_string()))
            .map(|value| value.filter(|text| !text.is_empty()));
    }
    if stored.starts_with(secrets_manager::INACTIVE_KEYRING_PREFIX) {
        return Err(SshAuthError::other(
            "Core SSH secret uses the inactive keyring prefix for this build",
        ));
    }
    Ok(Some(stored.to_string()))
}

pub fn store_core_secret(
    store: &dyn SecretStore,
    setting_key: &str,
    secret_value: &str,
) -> Result<String, SshAuthError> {
    let lookup_key = secrets_manager::secret_lookup_key("ssh", "core", setting_key);
    store
        .set(&lookup_key, secret_value)
        .map_err(|error| SshAuthError::other(error.to_string()))?;
    Ok(format!("{}{}", KEYRING_PREFIX, lookup_key))
}

pub fn get_core_secret(store: &dyn SecretStore, setting_key: &str) -> Result<Option<String>, SshAuthError> {
    let lookup_key = secrets_manager::secret_lookup_key("ssh", "core", setting_key);
    store
        .get(&lookup_key)
        .map_err(|error| SshAuthError::other(error.to_string()))
}

pub fn remove_core_secret(store: &dyn SecretStore, setting_key: &str) -> Result<(), SshAuthError> {
    let lookup_key = secrets_manager::secret_lookup_key("ssh", "core", setting_key);
    store
        .delete(&lookup_key)
        .map_err(|error| SshAuthError::other(error.to_string()))
}

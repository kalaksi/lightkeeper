/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use ssh2;

use super::ssh_auth::{accept_host_key, authenticate, check_known_hosts, resolve_secret_value, SshAuthError, SshAuthOptions};
use crate::configuration::CoreConnectionProfile;
use crate::module::PlatformInfo;
use crate::secrets_manager::KeyringSecretStore;

const SSH_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Absolute user-local paths for installing lightkeeper-core on the probed remote core host
/// (`~/.local/bin`, systemd user unit, data-dir socket). Used by the Install confirmation UI
/// and by the remote install path when copying/enabling the service.
#[derive(Clone, PartialEq, Eq)]
pub struct CoreInstallPlan {
    pub binary_path: String,
    pub unit_path: String,
    pub socket_path: String,
}

/// Result of probing a remote core host over SSH without requiring a live core socket.
#[derive(Clone, PartialEq, Eq)]
pub struct RemoteCoreHostProbe {
    pub platform: PlatformInfo,
    /// Absolute path if a core socket inode is present; otherwise `None`.
    pub socket_path: Option<String>,
    /// Absolute path if `lightkeeper-core` was found; otherwise `None`.
    pub binary_path: Option<String>,
    pub install_plan: CoreInstallPlan,
}

pub struct Ssh2DirectStreamLocalTransport {
    session: ssh2::Session,
    channel: ssh2::Channel,
}

impl Ssh2DirectStreamLocalTransport {
    pub fn start(profile: &CoreConnectionProfile, cancel: &AtomicBool) -> Result<Self, SshAuthError> {
        let session = connect_remote_core_session(profile, cancel)?;
        let remote_socket = match &profile.remote_socket_path {
            Some(path) => validate_remote_socket_path(path).map_err(SshAuthError::other)?,
            None => discover_remote_core_socket_path(&session)?,
        };

        check_cancel(cancel)?;
        let channel = session
            .channel_direct_streamlocal(&remote_socket, None)
            .map_err(|error| SshAuthError::other(format!("Failed to open direct-streamlocal to {}: {}", remote_socket, error)))?;

        Ok(Ssh2DirectStreamLocalTransport { session, channel })
    }

    pub fn channel(&self) -> ssh2::Channel {
        self.channel.clone()
    }

    pub fn set_timeout_ms(&self, timeout_ms: u32) {
        self.session.set_timeout(timeout_ms);
    }

    pub fn stop(&mut self) {
        let _ = self.channel.close();
        let _ = self.channel.wait_close();
        let _ = self.session.disconnect(None, "", None);
    }
}

impl Drop for Ssh2DirectStreamLocalTransport {
    fn drop(&mut self) {
        self.stop();
    }
}

/// SSH to the remote core host, collect platform info and core install presence (no streamlocal).
pub fn probe_remote_core_host(profile: &CoreConnectionProfile, cancel: &AtomicBool) -> Result<RemoteCoreHostProbe, SshAuthError> {
    let session = connect_remote_core_session(profile, cancel)?;
    let os_release = exec_command(&session, "cat /etc/os-release")?;
    let uname_machine = exec_command(&session, "uname -m")?;
    let platform = PlatformInfo::from_linux_probe(&os_release, &uname_machine);
    let discovered_socket_path = match &profile.remote_socket_path {
        Some(path) => validate_remote_socket_path(path).map_err(SshAuthError::other)?,
        None => discover_remote_core_socket_path(&session)?,
    };
    let socket_path = if remote_socket_exists(&session, &discovered_socket_path)? {
        Some(discovered_socket_path.clone())
    }
    else {
        None
    };
    let binary_path = find_core_binary(&session)?;
    let install_plan = resolve_install_plan(&session, &discovered_socket_path)?;

    Ok(RemoteCoreHostProbe {
        platform,
        socket_path,
        binary_path,
        install_plan,
    })
}

/// Reconnects, checks the presented key matches `expected_key_id`, and writes known_hosts.
pub fn accept_remote_core_host_key(
    profile: &CoreConnectionProfile,
    expected_key_id: &str,
    cancel: &AtomicBool,
) -> Result<(), SshAuthError> {
    let session = connect_remote_core_session_skip_host_key(profile, cancel)?;
    let port = profile.port.unwrap_or(22);
    accept_host_key(&session, &profile.host, port, expected_key_id, None)
}

fn resolve_install_plan(session: &ssh2::Session, socket_path: &str) -> Result<CoreInstallPlan, SshAuthError> {
    let home = exec_command(session, "printf '%s' \"$HOME\"")?;
    let home = home.trim();
    if home.is_empty() || !home.starts_with('/') {
        return Err(SshAuthError::other("Remote HOME is missing or invalid"));
    }

    Ok(CoreInstallPlan {
        binary_path: format!("{}/.local/bin/lightkeeper-core", home),
        unit_path: format!("{}/.config/systemd/user/lightkeeper-core.service", home),
        socket_path: socket_path.to_string(),
    })
}

fn connect_remote_core_session(profile: &CoreConnectionProfile, cancel: &AtomicBool) -> Result<ssh2::Session, SshAuthError> {
    let session = connect_remote_core_tcp_and_handshake(profile, cancel)?;
    let port = profile.port.unwrap_or(22);

    check_cancel(cancel)?;
    check_known_hosts(&session, &profile.host, port, None)?;

    check_cancel(cancel)?;
    let auth = build_auth_options(profile)?;
    authenticate(&session, &auth)?;
    check_cancel(cancel)?;
    Ok(session)
}

fn connect_remote_core_session_skip_host_key(
    profile: &CoreConnectionProfile,
    cancel: &AtomicBool,
) -> Result<ssh2::Session, SshAuthError> {
    connect_remote_core_tcp_and_handshake(profile, cancel)
}

fn connect_remote_core_tcp_and_handshake(profile: &CoreConnectionProfile, cancel: &AtomicBool) -> Result<ssh2::Session, SshAuthError> {
    if !profile.is_configured() {
        return Err(SshAuthError::other("Core connection profile has no SSH host"));
    }
    check_cancel(cancel)?;

    let port = profile.port.unwrap_or(22);
    let username = profile
        .username
        .clone()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(default_ssh_username);
    if username.starts_with('-') {
        return Err(SshAuthError::other("Invalid SSH username"));
    }

    let mut addresses = format!("{}:{}", profile.host, port)
        .to_socket_addrs()
        .map_err(|error| SshAuthError::other(format!("Failed to resolve {}: {}", profile.host, error)))?;
    let address = addresses
        .next()
        .ok_or_else(|| SshAuthError::other(format!("Failed to resolve {}: no addresses", profile.host)))?;

    check_cancel(cancel)?;
    let tcp = TcpStream::connect_timeout(&address, SSH_CONNECT_TIMEOUT)
        .map_err(|error| SshAuthError::other(format!("SSH TCP connect failed: {}", error)))?;
    tcp.set_nodelay(true).map_err(|error| SshAuthError::other(error.to_string()))?;

    check_cancel(cancel)?;
    let mut session = ssh2::Session::new().map_err(|error| SshAuthError::other(error.to_string()))?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|error| SshAuthError::other(format!("SSH handshake failed: {}", error)))?;
    Ok(session)
}

fn build_auth_options(profile: &CoreConnectionProfile) -> Result<SshAuthOptions, SshAuthError> {
    let store = KeyringSecretStore;
    let username = profile
        .username
        .clone()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(default_ssh_username);
    let password = match &profile.password {
        Some(stored) => resolve_secret_value(&store, stored)?,
        None => None,
    };
    let private_key_passphrase = match &profile.private_key_passphrase {
        Some(stored) => resolve_secret_value(&store, stored)?,
        None => None,
    };

    Ok(SshAuthOptions {
        username,
        password,
        private_key_path: profile.private_key_path.clone().filter(|path| !path.is_empty()),
        private_key_passphrase,
        agent_key_identifier: profile.agent_key_identifier.clone().filter(|id| !id.is_empty()),
    })
}

fn remote_socket_exists(session: &ssh2::Session, socket_path: &str) -> Result<bool, SshAuthError> {
    // Path already validated as absolute without shell metacharacters beyond '/'.
    let command = format!(
        "if [ -S '{}' ]; then printf yes; else printf no; fi",
        socket_path.replace('\'', "'\\''"),
    );
    let output = exec_command(session, &command)?;
    Ok(output.trim() == "yes")
}

fn find_core_binary(session: &ssh2::Session) -> Result<Option<String>, SshAuthError> {
    let command = concat!(
        "if command -v lightkeeper-core >/dev/null 2>&1; then command -v lightkeeper-core; ",
        "elif [ -x \"${HOME}/.local/bin/lightkeeper-core\" ]; then ",
        "printf '%s\\n' \"${HOME}/.local/bin/lightkeeper-core\"; ",
        "elif [ -x /usr/bin/lightkeeper-core ]; then printf '%s\\n' /usr/bin/lightkeeper-core; ",
        "fi",
    );
    let output = exec_command(session, command)?;
    Ok(parse_core_binary_path(&output))
}

fn exec_command(session: &ssh2::Session, command: &str) -> Result<String, SshAuthError> {
    let mut channel = session
        .channel_session()
        .map_err(|error| SshAuthError::other(format!("Failed to open SSH session channel: {}", error)))?;
    channel
        .exec(command)
        .map_err(|error| SshAuthError::other(format!("Failed to run remote command: {}", error)))?;

    let mut output = String::new();
    channel
        .read_to_string(&mut output)
        .map_err(|error| SshAuthError::other(format!("Failed to read remote command output: {}", error)))?;
    let _ = channel.wait_close();
    let status = channel.exit_status().unwrap_or(-1);
    if status != 0 {
        return Err(SshAuthError::other(format!("Remote command failed (exit {}): {}", status, command)));
    }
    Ok(output)
}

/// Validates an explicit `remote_socket_path` from the connection profile.
pub fn resolve_remote_socket_path(profile: &CoreConnectionProfile) -> Result<String, String> {
    let Some(path) = &profile.remote_socket_path
    else {
        return Err(String::from("Remote core socket path is not set"));
    };
    validate_remote_socket_path(path)
}

fn validate_remote_socket_path(path: &str) -> Result<String, String> {
    if path.is_empty() {
        return Err(String::from("Remote core socket path is empty"));
    }
    if !path.starts_with('/') {
        return Err(String::from("Remote core socket path must be absolute"));
    }
    if path.contains('\0') || path.contains('\n') {
        return Err(String::from("Remote core socket path contains invalid characters"));
    }
    Ok(path.to_string())
}

fn discover_remote_core_socket_path(session: &ssh2::Session) -> Result<String, SshAuthError> {
    // Same rules as host `get_data_dir()` for a non-Flatpak remote core host (HOME is reliable over SSH).
    let discovery_command = concat!(
        "if [ ! -z \"${XDG_DATA_HOME}\" ]; then ",
        "printf '%s\\n' \"${XDG_DATA_HOME}/lightkeeper/core.sock\"; ",
        "elif [ ! -z \"${HOME}\" ]; then ",
        "printf '%s\\n' \"${HOME}/.local/share/lightkeeper/core.sock\"; ",
        "else exit 1; fi",
    );

    let output = exec_command(session, discovery_command)
        .map_err(|error| SshAuthError::other(format!("Remote socket discovery failed: {}", error.to_ui_message())))?;
    parse_discovered_socket_path(&output).map_err(SshAuthError::other)
}

pub fn parse_discovered_socket_path(output: &str) -> Result<String, String> {
    let path = output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .ok_or_else(|| String::from("Remote socket discovery returned no path"))?;
    validate_remote_socket_path(path)
}

/// Parses binary-discovery command output (first non-empty line).
pub fn parse_core_binary_path(output: &str) -> Option<String> {
    output.lines().map(str::trim).find(|line| !line.is_empty()).map(str::to_string)
}

fn default_ssh_username() -> String {
    std::env::var("USER").unwrap_or_else(|_| String::from("root"))
}

fn check_cancel(cancel: &AtomicBool) -> Result<(), SshAuthError> {
    if cancel.load(Ordering::SeqCst) {
        Err(SshAuthError::other("SSH core connection cancelled"))
    }
    else {
        Ok(())
    }
}

/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use ssh2;

use crate::configuration::CoreConnectionProfile;
use crate::file_handler;
use crate::utils::sha256;

const SSH_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

pub struct Ssh2DirectStreamLocalTransport {
    session: ssh2::Session,
    channel: ssh2::Channel,
}

impl Ssh2DirectStreamLocalTransport {
    pub fn start(profile: &CoreConnectionProfile, cancel: &AtomicBool) -> Result<Self, String> {
        if !profile.is_configured() {
            return Err(String::from("Core connection profile has no SSH host"));
        }
        check_cancel(cancel)?;

        let port = profile.port.unwrap_or(22);
        let username = profile
            .username
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(default_ssh_username);
        if username.starts_with('-') {
            return Err(String::from("Invalid SSH username"));
        }

        let mut addresses = format!("{}:{}", profile.host, port)
            .to_socket_addrs()
            .map_err(|error| format!("Failed to resolve {}: {}", profile.host, error))?;
        let address = addresses
            .next()
            .ok_or_else(|| format!("Failed to resolve {}: no addresses", profile.host))?;

        check_cancel(cancel)?;
        let tcp = TcpStream::connect_timeout(&address, SSH_CONNECT_TIMEOUT)
            .map_err(|error| format!("SSH TCP connect failed: {}", error))?;
        tcp.set_nodelay(true).map_err(|error| error.to_string())?;

        check_cancel(cancel)?;
        let mut session = ssh2::Session::new().map_err(|error| error.to_string())?;
        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|error| format!("SSH handshake failed: {}", error))?;

        check_cancel(cancel)?;
        verify_host_key(&session, &profile.host, port)?;
        authenticate_agent(&session, &username)?;

        check_cancel(cancel)?;
        let remote_socket = match &profile.remote_socket_path {
            Some(path) => validate_remote_socket_path(path)?,
            None => discover_remote_core_socket_path(&session)?,
        };

        check_cancel(cancel)?;
        let channel = session
            .channel_direct_streamlocal(&remote_socket, None)
            .map_err(|error| {
                format!(
                    "Failed to open direct-streamlocal to {}: {}",
                    remote_socket, error
                )
            })?;

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

/// Validates an explicit `remote_socket_path` from the connection profile.
pub fn resolve_remote_socket_path(profile: &CoreConnectionProfile) -> Result<String, String> {
    let Some(path) = &profile.remote_socket_path else {
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

fn discover_remote_core_socket_path(session: &ssh2::Session) -> Result<String, String> {
    // Same rules as host `get_data_dir()` for a non-Flatpak admin host (HOME is reliable over SSH).
    let discovery_command = concat!(
        "if [ ! -z \"${XDG_DATA_HOME}\" ]; then ",
        "printf '%s\\n' \"${XDG_DATA_HOME}/lightkeeper/core.sock\"; ",
        "elif [ ! -z \"${HOME}\" ]; then ",
        "printf '%s\\n' \"${HOME}/.local/share/lightkeeper/core.sock\"; ",
        "else exit 1; fi",
    );

    let mut channel = session
        .channel_session()
        .map_err(|error| format!("Failed to open SSH session for socket discovery: {}", error))?;
    channel
        .exec(discovery_command)
        .map_err(|error| format!("Failed to run remote socket discovery: {}", error))?;

    let mut output = String::new();
    channel
        .read_to_string(&mut output)
        .map_err(|error| format!("Failed to read remote socket discovery output: {}", error))?;
    let _ = channel.wait_close();
    let status = channel.exit_status().unwrap_or(-1);
    if status != 0 {
        return Err(format!(
            "Remote socket discovery failed (exit {})",
            status
        ));
    }

    parse_discovered_socket_path(&output)
}

pub fn parse_discovered_socket_path(output: &str) -> Result<String, String> {
    let path = output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .ok_or_else(|| String::from("Remote socket discovery returned no path"))?;
    validate_remote_socket_path(path)
}

fn default_ssh_username() -> String {
    std::env::var("USER").unwrap_or_else(|_| String::from("root"))
}

fn check_cancel(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::SeqCst) {
        Err(String::from("SSH core connection cancelled"))
    }
    else {
        Ok(())
    }
}

fn authenticate_agent(session: &ssh2::Session, username: &str) -> Result<(), String> {
    let mut agent = session
        .agent()
        .map_err(|error| format!("Failed to open SSH agent: {}", error))?;
    agent
        .connect()
        .map_err(|error| format!("Failed to connect to SSH agent: {}", error))?;
    agent
        .list_identities()
        .map_err(|error| format!("Failed to list SSH agent identities: {}", error))?;

    for identity in agent.identities().map_err(|error| error.to_string())? {
        ::log::debug!("Trying core SSH auth with agent key \"{}\"", identity.comment());
        if agent.userauth(username, &identity).is_ok() && session.authenticated() {
            return Ok(());
        }
    }

    Err(String::from("Failed to authenticate to admin host with SSH agent"))
}

fn verify_host_key(session: &ssh2::Session, hostname: &str, port: u16) -> Result<(), String> {
    let known_hosts_path = known_hosts_path()?;
    let mut known_hosts = session
        .known_hosts()
        .map_err(|error| format!("Failed to initialize known_hosts: {}", error))?;
    known_hosts
        .read_file(&known_hosts_path, ssh2::KnownHostFileKind::OpenSSH)
        .map_err(|error| format!("Failed to read known_hosts: {}", error))?;

    let (key, _key_type) = session
        .host_key()
        .ok_or_else(|| String::from("SSH server did not present a host key"))?;

    match known_hosts.check_port(hostname, port, key) {
        ssh2::CheckResult::Match => Ok(()),
        ssh2::CheckResult::NotFound => Err(format!(
            "Host key for '{}' was not found in {}. Fingerprint SHA256: {}",
            hostname,
            known_hosts_path.display(),
            sha256::hash(key),
        )),
        ssh2::CheckResult::Mismatch => Err(format!(
            "Host key for '{}' HAS CHANGED (see {}). Fingerprint SHA256: {}",
            hostname,
            known_hosts_path.display(),
            sha256::hash(key),
        )),
        ssh2::CheckResult::Failure => Err(format!("Failed to check host key for '{}'", hostname)),
    }
}

fn known_hosts_path() -> Result<PathBuf, String> {
    let path = file_handler::get_config_dir().join("known_hosts");
    if !path.exists() {
        std::fs::File::create(&path).map_err(|error| {
            format!("Failed to create known_hosts {}: {}", path.display(), error)
        })?;
    }
    Ok(path)
}

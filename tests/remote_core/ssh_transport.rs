/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use lightkeeper::backend::ssh_transport::{
    parse_core_binary_path, parse_discovered_socket_path, resolve_remote_socket_path,
};
use lightkeeper::configuration::CoreConnectionProfile;
use lightkeeper::enums::CoreTransportPreference;
use lightkeeper::module::platform_info::{Architecture, Flavor, OperatingSystem, PlatformInfo};

fn sample_profile() -> CoreConnectionProfile {
    CoreConnectionProfile {
        host: String::from("admin.example"),
        port: Some(2222),
        username: Some(String::from("ops")),
        remote_socket_path: Some(String::from("/var/tmp/lightkeeper/core.sock")),
        transport: CoreTransportPreference::Ssh2DirectStreamLocal,
        auto_connect: false,
    }
}

#[test]
fn resolve_remote_socket_path_uses_override() {
    let profile = sample_profile();
    assert_eq!(
        resolve_remote_socket_path(&profile).unwrap(),
        "/var/tmp/lightkeeper/core.sock",
    );
}

#[test]
fn resolve_remote_socket_path_without_override_is_discovered_at_connect() {
    let profile = CoreConnectionProfile {
        host: String::from("admin.example"),
        ..CoreConnectionProfile::default()
    };
    let error = resolve_remote_socket_path(&profile).unwrap_err();
    assert!(error.contains("not set"), "unexpected error: {}", error);
}

#[test]
fn resolve_remote_socket_path_rejects_relative() {
    let mut profile = sample_profile();
    profile.remote_socket_path = Some(String::from("relative/core.sock"));
    let error = resolve_remote_socket_path(&profile).unwrap_err();
    assert!(error.contains("absolute"));
}

#[test]
fn resolve_remote_socket_path_rejects_invalid_characters() {
    let mut profile = sample_profile();
    profile.remote_socket_path = Some(String::from("/tmp/core\nsock"));
    let error = resolve_remote_socket_path(&profile).unwrap_err();
    assert!(error.contains("invalid characters"));
}

#[test]
fn parse_discovered_socket_path_uses_first_line() {
    assert_eq!(
        parse_discovered_socket_path("  /home/ops/.local/share/lightkeeper/core.sock\n").unwrap(),
        "/home/ops/.local/share/lightkeeper/core.sock",
    );
}

#[test]
fn parse_discovered_socket_path_rejects_relative() {
    let error = parse_discovered_socket_path("relative/core.sock\n").unwrap_err();
    assert!(error.contains("absolute"));
}

#[test]
fn parse_core_binary_path_uses_first_line() {
    assert_eq!(
        parse_core_binary_path("  /home/ops/.local/bin/lightkeeper-core\n").unwrap(),
        "/home/ops/.local/bin/lightkeeper-core",
    );
    assert!(parse_core_binary_path("\n").is_none());
}

#[test]
fn platform_info_from_linux_probe_matches_os_release() {
    let platform = PlatformInfo::from_linux_probe(
        "ID=fedora\nVERSION_ID=44\nVARIANT_ID=workstation\n",
        "x86_64\n",
    );
    assert!(matches!(platform.os, OperatingSystem::Linux));
    assert!(matches!(platform.os_flavor, Flavor::Fedora));
    assert!(matches!(platform.architecture, Architecture::X86_64));
    assert_eq!(platform.os_variant_id, "workstation");
}

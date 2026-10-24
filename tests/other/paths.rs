/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::path::PathBuf;

use lightkeeper::backend::ssh_auth::{authenticate, known_hosts_path, SshAuthOptions};
use lightkeeper::file_handler;

use crate::{unique_target_dir, EnvVarGuard};

#[test]
fn lightkeeper_config_dir_env_overrides_xdg() {
    let config_dir = unique_target_dir("lk-config-env");
    let _wrong_xdg = EnvVarGuard::set("XDG_CONFIG_HOME", unique_target_dir("wrong-xdg"));
    let _lk_config = EnvVarGuard::set("LIGHTKEEPER_CONFIG_DIR", &config_dir);

    assert_eq!(file_handler::get_config_dir(), config_dir);
}

#[test]
fn known_hosts_path_creates_missing_parents_under_config_dir() {
    let config_dir = unique_target_dir("lk-known-hosts").join("nested").join("config");
    assert!(!config_dir.exists());
    let _lk_config = EnvVarGuard::set("LIGHTKEEPER_CONFIG_DIR", &config_dir);
    let _clear_xdg = EnvVarGuard::remove("XDG_CONFIG_HOME");

    let path = known_hosts_path(None).unwrap();
    assert_eq!(path, config_dir.join("known_hosts"));
    assert!(path.is_file());
}

#[test]
fn authenticate_reports_missing_private_key_path() {
    let session = ssh2::Session::new().unwrap();
    let missing = unique_target_dir("missing-key-parent").join("no-such-key");
    let error = authenticate(
        &session,
        &SshAuthOptions {
            username: String::from("user"),
            private_key_path: Some(missing.to_string_lossy().to_string()),
            ..Default::default()
        },
    )
    .unwrap_err();

    let message = error.to_ui_message();
    assert!(message.contains("Private key file not found"), "message was: {}", message);
    assert!(message.contains(missing.to_string_lossy().as_ref()), "message was: {}", message);
}

#[test]
fn cache_dir_with_home_root_resolves_under_slash_cache() {
    let _clear_xdg = EnvVarGuard::remove("XDG_CACHE_HOME");
    let _home = EnvVarGuard::set("HOME", "/");

    assert_eq!(file_handler::get_cache_dir(), PathBuf::from("/.cache/lightkeeper"));
}

#[test]
fn cache_dir_respects_xdg_cache_home_even_with_home_root() {
    let cache_root = unique_target_dir("xdg-cache-home");
    let _home = EnvVarGuard::set("HOME", "/");
    let _xdg_cache = EnvVarGuard::set("XDG_CACHE_HOME", &cache_root);

    assert_eq!(file_handler::get_cache_dir(), cache_root.join("lightkeeper"));
}

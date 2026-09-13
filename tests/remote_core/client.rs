/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use lightkeeper::backend::{CoreConnectionState, RemoteCoreClient};
use lightkeeper::configuration::Configuration;
use lightkeeper::remote_core::runtime::CoreRuntime;
use lightkeeper::remote_core::server::{self, CoreListener};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join(format!(
        "lk-{}-{}-{}",
        prefix,
        std::process::id(),
        nanos
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn probe_succeeds_against_local_core_socket() {
    let _ = env_logger::Builder::from_default_env().is_test(true).try_init();

    let socket_dir = unique_temp_dir("core-probe-sock");
    let socket_path = socket_dir.join("core.sock");
    let listener = CoreListener::bind(socket_path.clone()).unwrap();

    let config_dir = unique_temp_dir("core-probe-config");
    let config_dir_str = config_dir.to_string_lossy().to_string();
    Configuration::write_initial_config(&config_dir).unwrap();
    let (main_config, hosts, _groups) = Configuration::read(&config_dir_str).unwrap();
    let mut runtime = CoreRuntime::new(&main_config, &hosts, config_dir_str).unwrap();

    let client_path = socket_path.clone();
    let client_thread = thread::spawn(move || {
        let client = RemoteCoreClient::new(client_path);
        client.probe().unwrap();
        assert_eq!(client.connection_state(), CoreConnectionState::Disconnected);
        assert!(!client.is_connected());
    });

    let stream = listener.recv().unwrap();
    // Probe disconnects right after handshake; InitialState send may see a broken pipe.
    let _ = server::run_claimed_remote_client_session(stream, &mut runtime, &listener.session_active_flag());
    client_thread.join().unwrap();

    drop(listener);
    let _ = fs::remove_dir_all(socket_dir);
    let _ = fs::remove_dir_all(config_dir);
}

#[test]
fn probe_fails_when_socket_missing() {
    let dir = unique_temp_dir("core-probe-missing");
    let socket_path = dir.join("missing.sock");
    let client = RemoteCoreClient::new(socket_path);
    let error = client.probe().unwrap_err();
    assert!(!error.is_empty());
    assert_eq!(client.connection_state(), CoreConnectionState::Failed);
    assert!(client.last_error().is_some());
    let _ = fs::remove_dir_all(dir);
}

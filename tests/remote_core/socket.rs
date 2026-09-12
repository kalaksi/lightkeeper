/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use lightkeeper::configuration::Configuration;
use lightkeeper::remote_core::protocol::{
    read_message, write_message, ClientMessage, RemoteErrorCode, ServerMessage, PROTOCOL_VERSION,
};
use lightkeeper::remote_core::runtime::CoreRuntime;
use lightkeeper::remote_core::server::{self, CoreListener};
use lightkeeper::remote_core::socket::{self, SOCKET_DIR_MODE, SOCKET_FILE_MODE};

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
fn prepare_socket_path_sets_private_dir_mode() {
    let dir = unique_temp_dir("core-sock-dir");
    let socket_path = dir.join("nested").join("core.sock");

    socket::prepare_socket_path(&socket_path).unwrap();

    let parent = socket_path.parent().unwrap();
    let mode = fs::metadata(parent).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, SOCKET_DIR_MODE);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn remove_stale_socket_rejects_regular_file() {
    let dir = unique_temp_dir("core-sock-file");
    let path = dir.join("not-a-socket");
    fs::write(&path, b"nope").unwrap();

    let error = socket::remove_stale_socket(&path).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn bind_sets_socket_file_mode() {
    let dir = unique_temp_dir("core-sock-bind");
    let socket_path = dir.join("core.sock");
    socket::prepare_socket_path(&socket_path).unwrap();
    let _listener = UnixListener::bind(&socket_path).unwrap();
    socket::set_socket_permissions(&socket_path).unwrap();

    let mode = fs::metadata(&socket_path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, SOCKET_FILE_MODE);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn second_client_is_rejected_while_first_is_connected() {
    let _ = env_logger::Builder::from_default_env().is_test(true).try_init();

    let socket_dir = unique_temp_dir("core-server-sock");
    let socket_path = socket_dir.join("core.sock");
    let listener = CoreListener::bind(socket_path.clone()).unwrap();

    let connect_path = socket_path.clone();
    let first_client = thread::spawn(move || {
        let stream = UnixStream::connect(&connect_path).unwrap();
        // Keep the connection open until the test releases the claim.
        thread::park();
        drop(stream);
    });

    let _first_stream = listener.recv().unwrap();

    let mut second = UnixStream::connect(&socket_path).unwrap();
    second
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write_message(
        &mut second,
        &ClientMessage::Connect {
            protocol_version: PROTOCOL_VERSION,
        },
    )
    .unwrap();

    let mut saw_busy_error = false;
    let read_deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < read_deadline {
        match read_message::<ServerMessage, _>(&mut second) {
            Ok(ServerMessage::Error { message, code, .. }) => {
                assert!(message.contains("already connected"), "unexpected error: {}", message);
                assert_eq!(code, RemoteErrorCode::Busy);
                saw_busy_error = true;
                break;
            }
            Ok(_) => panic!("unexpected message while busy"),
            Err(error) if error.kind() == std::io::ErrorKind::TimedOut => break,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(error) => panic!("read failed: {}", error),
        }
    }
    assert!(saw_busy_error, "second client did not receive busy rejection");

    listener.release_session_claim();
    first_client.thread().unpark();
    first_client.join().unwrap();
    drop(listener);
    let _ = fs::remove_dir_all(socket_dir);
}

#[test]
fn failed_handshake_releases_session_claim() {
    let _ = env_logger::Builder::from_default_env().is_test(true).try_init();

    let socket_dir = unique_temp_dir("core-handshake-release");
    let socket_path = socket_dir.join("core.sock");
    let listener = CoreListener::bind(socket_path.clone()).unwrap();

    let config_dir = unique_temp_dir("core-handshake-config");
    let config_dir_str = config_dir.to_string_lossy().to_string();
    Configuration::write_initial_config(&config_dir).unwrap();
    let (main_config, hosts, _groups) = Configuration::read(&config_dir_str).unwrap();
    let mut runtime = CoreRuntime::new(&main_config, &hosts, config_dir_str).unwrap();

    let connect_path = socket_path.clone();
    let bad_client = thread::spawn(move || {
        let mut stream = UnixStream::connect(&connect_path).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        write_message(
            &mut stream,
            &ClientMessage::Connect {
                protocol_version: PROTOCOL_VERSION.wrapping_add(1),
            },
        )
        .unwrap();

        match read_message::<ServerMessage, _>(&mut stream).unwrap() {
            ServerMessage::Error { code, .. } => assert_eq!(code, RemoteErrorCode::UnsupportedVersion),
            _ => panic!("expected unsupported version error"),
        }
    });

    let stream = listener.recv().unwrap();
    assert!(
        *listener.session_active_flag().lock().unwrap(),
        "accept thread should claim the session before handoff"
    );

    server::run_claimed_remote_client_session(stream, &mut runtime, &listener.session_active_flag()).unwrap();
    bad_client.join().unwrap();

    assert!(
        !*listener.session_active_flag().lock().unwrap(),
        "failed handshake must release the session claim"
    );

    let connect_path = socket_path.clone();
    let second_client = thread::spawn(move || UnixStream::connect(&connect_path).unwrap());
    let accepted = listener.recv().unwrap();
    second_client.join().unwrap();
    assert!(
        *listener.session_active_flag().lock().unwrap(),
        "a new client should be able to claim after recovery"
    );
    drop(accepted);
    listener.release_session_claim();
    drop(listener);
    let _ = fs::remove_dir_all(socket_dir);
    let _ = fs::remove_dir_all(config_dir);
}

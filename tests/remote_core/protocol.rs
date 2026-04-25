/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use lightkeeper::remote_core::protocol::{
    read_message, write_message, ClientMessage, ServerMessage, PROTOCOL_VERSION,
};

#[test]
fn framed_message_roundtrip_works() {
    let mut buffer = Vec::new();
    let message = ClientMessage::Connect {
        protocol_version: PROTOCOL_VERSION,
    };

    write_message(&mut buffer, &message).unwrap();

    let decoded: ClientMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ClientMessage::Connect { protocol_version } => assert_eq!(protocol_version, PROTOCOL_VERSION),
        _ => panic!("Invalid message"),
    }
}

#[test]
fn server_message_roundtrip_works() {
    let mut buffer = Vec::new();
    let message = ServerMessage::ExecuteCommand {
        request_id: 7,
        invocation_id: 42,
    };

    write_message(&mut buffer, &message).unwrap();

    let decoded: ServerMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ServerMessage::ExecuteCommand {
            request_id,
            invocation_id,
        } => {
            assert_eq!(request_id, 7);
            assert_eq!(invocation_id, 42);
        }
        _ => panic!("Invalid message"),
    }
}

#[test]
fn has_cached_file_changed_client_roundtrip() {
    let mut buffer = Vec::new();
    let message = ClientMessage::HasCachedFileChanged {
        request_id: 9,
        host_id: "h1".to_string(),
        remote_file_path: "/etc/unit".to_string(),
        content_hash: "ab".repeat(32),
    };

    write_message(&mut buffer, &message).unwrap();

    let decoded: ClientMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ClientMessage::HasCachedFileChanged {
            request_id,
            host_id,
            remote_file_path,
            content_hash,
        } => {
            assert_eq!(request_id, 9);
            assert_eq!(host_id, "h1");
            assert_eq!(remote_file_path, "/etc/unit");
            assert_eq!(content_hash.len(), 64);
        }
        _ => panic!("Invalid message"),
    }
}

#[test]
fn write_cached_file_result_server_roundtrip() {
    let mut buffer = Vec::new();
    let message = ServerMessage::WriteCachedFileResult { request_id: 11 };

    write_message(&mut buffer, &message).unwrap();

    let decoded: ServerMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ServerMessage::WriteCachedFileResult { request_id } => assert_eq!(request_id, 11),
        _ => panic!("Invalid message"),
    }
}

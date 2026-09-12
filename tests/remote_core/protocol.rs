/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use lightkeeper::remote_core::protocol::{
    read_message, write_message, ClientMessage, ServerMessage, MAX_FRAME_SIZE, PROTOCOL_VERSION,
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

#[test]
fn trailing_bytes_in_frame_are_rejected() {
    let mut buffer = Vec::new();
    let message = ClientMessage::Connect {
        protocol_version: PROTOCOL_VERSION,
    };
    write_message(&mut buffer, &message).unwrap();

    // Inflate the length prefix and append junk after a valid payload.
    let payload_len = u32::from_be_bytes(buffer[0..4].try_into().unwrap()) as usize;
    let new_len = (payload_len + 4) as u32;
    buffer[0..4].copy_from_slice(&new_len.to_be_bytes());
    buffer.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

    match read_message::<ClientMessage, _>(&mut buffer.as_slice()) {
        Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::InvalidData),
        Ok(_) => panic!("expected trailing bytes to be rejected"),
    }
}

#[test]
fn zero_and_oversized_frames_are_rejected() {
    let zero = 0_u32.to_be_bytes();
    match read_message::<ClientMessage, _>(&mut zero.as_slice()) {
        Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::InvalidData),
        Ok(_) => panic!("expected zero-length frame to be rejected"),
    }

    let oversized = ((MAX_FRAME_SIZE as u32) + 1).to_be_bytes();
    match read_message::<ClientMessage, _>(&mut oversized.as_slice()) {
        Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::InvalidData),
        Ok(_) => panic!("expected oversized frame to be rejected"),
    }
}

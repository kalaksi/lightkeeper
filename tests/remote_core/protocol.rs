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


#[test]
fn initial_state_roundtrip_works() {
    use lightkeeper::frontend::DisplayData;
    use std::collections::HashMap;

    let message = ServerMessage::InitialState(DisplayData {
        hosts: HashMap::new(),
        all_monitor_names: vec![],
        table_headers: vec![],
    });
    let mut buffer = Vec::new();
    write_message(&mut buffer, &message).unwrap();
    let decoded: ServerMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ServerMessage::InitialState(data) => assert!(data.hosts.is_empty()),
        _ => panic!("expected InitialState"),
    }
}

#[test]
fn initial_state_with_hosts_roundtrip_works() {
    use lightkeeper::configuration::Configuration;

    let dir = crate::unique_target_dir("initial-state");
    Configuration::write_initial_config(&dir).unwrap();
    let config_dir = dir.to_string_lossy().to_string();
    let (main_config, hosts, _groups) = Configuration::read(&config_dir).unwrap();
    let runtime = crate::test_core_runtime(&main_config, &hosts, config_dir);
    let display_data = runtime.core.host_manager.borrow().get_display_data();

    let message = ServerMessage::InitialState(display_data);
    let mut buffer = Vec::new();
    write_message(&mut buffer, &message).unwrap();
    let decoded: ServerMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ServerMessage::InitialState(_) => {}
        _ => panic!("expected InitialState"),
    }
}

#[test]
fn build_file_transfer_command_client_roundtrip() {
    let mut buffer = Vec::new();
    let message = ClientMessage::FileTransferPlan {
        request_id: 11,
        host_id: "h1".to_string(),
        command_id: "_internal-filebrowser-upload".to_string(),
        remote_path: "/tmp/dest".to_string(),
    };

    write_message(&mut buffer, &message).unwrap();
    let decoded: ClientMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ClientMessage::FileTransferPlan {
            request_id,
            host_id,
            command_id,
            remote_path,
        } => {
            assert_eq!(request_id, 11);
            assert_eq!(host_id, "h1");
            assert_eq!(command_id, "_internal-filebrowser-upload");
            assert_eq!(remote_path, "/tmp/dest");
        }
        _ => panic!("Invalid message"),
    }
}

#[test]
fn file_transfer_command_result_server_roundtrip() {
    use lightkeeper::module::command::internal::filebrowser::download::RsyncSsh;
    use lightkeeper::remote_core::protocol::{FileTransferDirection, FileTransferPlan};
    use std::collections::HashMap;

    let mut buffer = Vec::new();
    let message = ServerMessage::FileTransferPlanResult {
        request_id: 12,
        plan: FileTransferPlan {
            invocation_id: 99,
            direction: FileTransferDirection::Download,
            address: "host".to_string(),
            remote_path: "/tmp/a".to_string(),
            ssh: RsyncSsh::new(&HashMap::from([("port".to_string(), "2222".to_string())])),
            use_sudo_rsync: true,
        },
    };

    write_message(&mut buffer, &message).unwrap();
    let decoded: ServerMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ServerMessage::FileTransferPlanResult { request_id, plan } => {
            assert_eq!(request_id, 12);
            assert_eq!(plan.invocation_id, 99);
            assert_eq!(plan.direction, FileTransferDirection::Download);
            assert_eq!(plan.address, "host");
            assert_eq!(plan.remote_path, "/tmp/a");
            assert_eq!(plan.ssh.port, 2222);
            assert!(plan.use_sudo_rsync);
        }
        _ => panic!("Invalid message"),
    }
}

#[test]
fn build_terminal_command_client_roundtrip() {
    let mut buffer = Vec::new();
    let message = ClientMessage::BuildTerminalCommand {
        request_id: 3,
        host_id: "h1".to_string(),
        command_id: "linux-shell".to_string(),
        parameters: vec![],
    };

    write_message(&mut buffer, &message).unwrap();
    let decoded: ClientMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ClientMessage::BuildTerminalCommand {
            request_id,
            host_id,
            command_id,
            parameters,
        } => {
            assert_eq!(request_id, 3);
            assert_eq!(host_id, "h1");
            assert_eq!(command_id, "linux-shell");
            assert!(parameters.is_empty());
        }
        _ => panic!("Invalid message"),
    }
}

#[test]
fn shell_command_result_server_roundtrip() {
    let mut buffer = Vec::new();
    let message = ServerMessage::ShellCommandResult {
        request_id: 4,
        arguments: vec![String::from("ssh"), String::from("-t"), String::from("host")],
    };

    write_message(&mut buffer, &message).unwrap();
    let decoded: ServerMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ServerMessage::ShellCommandResult { request_id, arguments } => {
            assert_eq!(request_id, 4);
            assert_eq!(arguments, vec![String::from("ssh"), String::from("-t"), String::from("host")]);
        }
        _ => panic!("Invalid message"),
    }
}

#[test]
fn get_metrics_client_roundtrip() {
    let mut buffer = Vec::new();
    let message = ClientMessage::GetMetrics {
        request_id: 5,
        host_id: "h1".to_string(),
        monitor_id: "load".to_string(),
        start_time: 10,
        end_time: 20,
    };

    write_message(&mut buffer, &message).unwrap();
    let decoded: ClientMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ClientMessage::GetMetrics {
            request_id,
            host_id,
            monitor_id,
            start_time,
            end_time,
        } => {
            assert_eq!(request_id, 5);
            assert_eq!(host_id, "h1");
            assert_eq!(monitor_id, "load");
            assert_eq!(start_time, 10);
            assert_eq!(end_time, 20);
        }
        _ => panic!("Invalid message"),
    }
}

#[test]
fn chart_and_metrics_query_server_roundtrip() {
    use lightkeeper::metrics::lmserver::LMSResponse;
    use std::collections::HashMap;

    let mut buffer = Vec::new();
    let message = ServerMessage::Chart(LMSResponse {
        request_id: 6,
        lag: 1,
        metrics: HashMap::new(),
        alerts: Vec::new(),
        errors: Vec::new(),
    });
    write_message(&mut buffer, &message).unwrap();
    let decoded: ServerMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ServerMessage::Chart(response) => assert_eq!(response.request_id, 6),
        _ => panic!("Invalid message"),
    }

    let mut buffer = Vec::new();
    let message = ServerMessage::MetricsQuery {
        request_id: 7,
        invocation_id: 8,
    };
    write_message(&mut buffer, &message).unwrap();
    let decoded: ServerMessage = read_message(&mut buffer.as_slice()).unwrap();
    match decoded {
        ServerMessage::MetricsQuery {
            request_id,
            invocation_id,
        } => {
            assert_eq!(request_id, 7);
            assert_eq!(invocation_id, 8);
        }
        _ => panic!("Invalid message"),
    }
}

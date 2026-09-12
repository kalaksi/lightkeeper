/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use lightkeeper::remote_core::protocol::{read_message, write_message, ServerMessage, MAX_FRAME_SIZE};

#[test]
fn server_config_yaml_payload_bincode_roundtrip() {
    let msg = ServerMessage::Config {
        request_id: 1,
        main_yml: String::from("preferences:\n  show_charts: false\n"),
        hosts_yml: String::from("hosts: {}\n"),
        groups_yml: String::from("groups: {}\n"),
    };
    let mut buffer = Vec::new();
    write_message(&mut buffer, &msg).unwrap();
    assert!(buffer.len() < MAX_FRAME_SIZE + 4);
    let _: ServerMessage = read_message(&mut buffer.as_slice()).unwrap();
}

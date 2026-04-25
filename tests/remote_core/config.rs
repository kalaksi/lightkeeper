/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use lightkeeper::remote_core::protocol::{ServerMessage, MAX_FRAME_SIZE};

#[test]
fn server_config_yaml_payload_bincode_roundtrip() {
    let msg = ServerMessage::Config {
        request_id: 1,
        main_yml: String::from("preferences:\n  show_charts: false\n"),
        hosts_yml: String::from("hosts: {}\n"),
        groups_yml: String::from("groups: {}\n"),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    assert!(bytes.len() < MAX_FRAME_SIZE);
    let _: ServerMessage = bincode::deserialize(&bytes).unwrap();
}

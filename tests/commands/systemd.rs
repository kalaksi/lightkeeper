/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use lightkeeper::module::*;
use lightkeeper::module::command::*;
use lightkeeper::module::command::systemd;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;

use crate::{CommandTestHarness, StubSsh2};


#[test]
fn test_start_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "systemctl" "start" "test-service.service""#,
            "", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Start::get_metadata(), systemd::service::Start::new_command_module),
    );

    let module_id = systemd::service::Start::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["test-service.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Info);
        assert_eq!(result.message, "");
    });
}

#[test]
fn test_start_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "systemctl" "start" "nonexistent.service""#,
            "Failed to start nonexistent.service: Unit nonexistent.service not found.", 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Start::get_metadata(), systemd::service::Start::new_command_module),
    );

    let module_id = systemd::service::Start::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["nonexistent.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Error);
        assert!(result.error.contains("Failed to start") || result.message.contains("Failed to start"));
    });
}

#[test]
fn test_stop_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "systemctl" "stop" "test-service.service""#,
            "", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Stop::get_metadata(), systemd::service::Stop::new_command_module),
    );

    let module_id = systemd::service::Stop::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["test-service.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Info);
        assert_eq!(result.message, "");
    });
}

#[test]
fn test_stop_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "systemctl" "stop" "nonexistent.service""#,
            "Failed to stop nonexistent.service: Unit nonexistent.service not loaded.", 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Stop::get_metadata(), systemd::service::Stop::new_command_module),
    );

    let module_id = systemd::service::Stop::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["nonexistent.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Error);
        assert!(result.error.contains("Failed to stop") || result.message.contains("Failed to stop"));
    });
}

#[test]
fn test_restart_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "systemctl" "restart" "test-service.service""#,
            "", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Restart::get_metadata(), systemd::service::Restart::new_command_module),
    );

    let module_id = systemd::service::Restart::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["test-service.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Info);
        assert_eq!(result.message, "");
    });
}

#[test]
fn test_restart_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "systemctl" "restart" "nonexistent.service""#,
            "Failed to restart nonexistent.service: Unit nonexistent.service not found.", 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Restart::get_metadata(), systemd::service::Restart::new_command_module),
    );

    let module_id = systemd::service::Restart::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["nonexistent.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Error);
        assert!(result.error.contains("Failed to restart") || result.message.contains("Failed to restart"));
    });
}

#[test]
fn test_mask_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "systemctl" "mask" "test-service.service""#,
            "", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Mask::get_metadata(), systemd::service::Mask::new_command_module),
    );

    let module_id = systemd::service::Mask::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["test-service.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Info);
        assert_eq!(result.message, "");
    });
}

#[test]
fn test_mask_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "systemctl" "mask" "nonexistent.service""#,
            "Failed to mask unit: Unit nonexistent.service not found.", 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Mask::get_metadata(), systemd::service::Mask::new_command_module),
    );

    let module_id = systemd::service::Mask::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["nonexistent.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Error);
        assert!(result.error.contains("Failed to mask") || result.message.contains("Failed to mask"));
    });
}

#[test]
fn test_unmask_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "systemctl" "unmask" "test-service.service""#,
            "", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Unmask::get_metadata(), systemd::service::Unmask::new_command_module),
    );

    let module_id = systemd::service::Unmask::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["test-service.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Info);
        assert_eq!(result.message, "");
    });
}

#[test]
fn test_unmask_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "systemctl" "unmask" "nonexistent.service""#,
            "Failed to unmask unit: Unit nonexistent.service not found.", 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Unmask::get_metadata(), systemd::service::Unmask::new_command_module),
    );

    let module_id = systemd::service::Unmask::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["nonexistent.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Error);
        assert!(result.error.contains("Failed to unmask") || result.message.contains("Failed to unmask"));
    });
}

#[test]
fn test_logs_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        let output = concat!(
            r#"{"PRIORITY":"6","_HOSTNAME":"hostname","SYSLOG_IDENTIFIER":"test-service","#,
            r#""_PID":"1234","__REALTIME_TIMESTAMP":"1733047200000000","#,
            r#""MESSAGE":"Starting test-service"}"#,
            "\n",
            r#"{"PRIORITY":"6","_HOSTNAME":"hostname","SYSLOG_IDENTIFIER":"test-service","#,
            r#""_PID":"1234","__REALTIME_TIMESTAMP":"1733047201000000","#,
            r#""MESSAGE":"test-service started successfully"}"#,
            "\n",
            r#"{"PRIORITY":"6","_HOSTNAME":"hostname","SYSLOG_IDENTIFIER":"test-service","#,
            r#""_PID":"1234","__REALTIME_TIMESTAMP":"1733047500000000","#,
            r#""MESSAGE":"Processing request"}"#,
        );
        StubSsh2::new(
            r#""sudo" "journalctl" "-q" "-o" "json" "-u" "test-service.service" "-n" "1000""#,
            output,
            0,
        )
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Logs::get_metadata(), systemd::service::Logs::new_command_module),
    );

    let module_id = systemd::service::Logs::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["test-service.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
        assert!(result.message.contains("test-service"));
        assert!(result.message.contains("color:#c9b458"));
    });
}

#[test]
fn test_logs_with_parameters() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        let output = concat!(
            r#"{"PRIORITY":"6","_HOSTNAME":"hostname","SYSLOG_IDENTIFIER":"test-service","#,
            r#""_PID":"1234","__REALTIME_TIMESTAMP":"1733047200000000","#,
            r#""MESSAGE":"Log entry 1"}"#,
            "\n",
            r#"{"PRIORITY":"6","_HOSTNAME":"hostname","SYSLOG_IDENTIFIER":"test-service","#,
            r#""_PID":"1234","__REALTIME_TIMESTAMP":"1733047201000000","#,
            r#""MESSAGE":"Log entry 2"}"#,
        );
        StubSsh2::new(
            r#""sudo" "journalctl" "-q" "-o" "json" "-u" "test-service.service" "-n" "1000""#,
            output,
            0,
        )
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Logs::get_metadata(), systemd::service::Logs::new_command_module),
    );

    let module_id = systemd::service::Logs::get_metadata().module_spec.id.clone();

    // Logs command accepts: service, start_time, end_time, page_number, page_size
    harness.execute_command(&module_id, vec![
        "test-service.service".to_string(),
        "".to_string(),
        "".to_string(),
        "1".to_string(),
        "1000".to_string(),
    ]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
        assert!(result.message.contains("Log entry"));
    });
}

#[test]
fn test_logs_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "journalctl" "-q" "-o" "json" "-u" "nonexistent.service" "-n" "1000""#,
            "No entries.", 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::service::Logs::get_metadata(), systemd::service::Logs::new_command_module),
    );

    let module_id = systemd::service::Logs::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["nonexistent.service".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Error);
    });
}


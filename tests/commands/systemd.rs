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
        // TODO: auto-generated responses, check or replace with actual
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
        // TODO: auto-generated responses, check or replace with actual
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
        // TODO: auto-generated responses, check or replace with actual
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
        // TODO: auto-generated responses, check or replace with actual
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
        // TODO: auto-generated responses, check or replace with actual
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
        // TODO: auto-generated responses, check or replace with actual
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
        // TODO: auto-generated responses, check or replace with actual
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
        // TODO: auto-generated responses, check or replace with actual
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
        // TODO: auto-generated responses, check or replace with actual
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
        // TODO: auto-generated responses, check or replace with actual
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
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "journalctl" "-q" "-u" "test-service.service" "-n" "1000""#,
            r#"Dec 01 10:00:00 hostname test-service[1234]: Starting test-service
Dec 01 10:00:01 hostname test-service[1234]: test-service started successfully
Dec 01 10:05:00 hostname test-service[1234]: Processing request"#, 0)
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
    });
}

#[test]
fn test_logs_with_parameters() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "journalctl" "-q" "-u" "test-service.service" "-n" "1000""#,
            r#"Dec 01 10:00:00 hostname test-service[1234]: Log entry 1
Dec 01 10:00:01 hostname test-service[1234]: Log entry 2"#, 0)
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
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "journalctl" "-q" "-u" "nonexistent.service" "-n" "1000""#,
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


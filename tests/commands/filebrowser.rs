/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use lightkeeper::enums::Criticality;
use lightkeeper::module::*;
use lightkeeper::module::command::CommandModule;
use lightkeeper::module::command::internal::FileBrowserChmod;
use lightkeeper::module::command::internal::filebrowser::du::parse_du_output;
use lightkeeper::module::command::internal::filebrowser::ls::parse_ls_output;
use lightkeeper::module::platform_info::*;

use crate::{CommandTestHarness, StubSsh2};

#[test]
fn test_ls_parse_preserves_special_bits() {
    let output = concat!(
        "total 8\n",
        "drwxrwsr-x 2 owner group 4096 2026-09-03 10:00 setgid directory\n",
        "-rwsr-xr-x+ 1 owner group 42 2026-09-03 10:01 acl-file\n",
        "drwxrwxrwt 2 root root 4096 2026-09-03 10:02 sticky\n",
    );

    let entries = parse_ls_output(output).unwrap();

    assert_eq!(entries[0]["permissions"], "drwxrwsr-x");
    assert_eq!(entries[0]["name"], "setgid directory");
    assert_eq!(entries[1]["permissions"], "-rwsr-xr-x+");
    assert_eq!(entries[2]["permissions"], "drwxrwxrwt");
}

#[test]
fn test_ls_parse_full_time_with_timezone() {
    let output = concat!(
        "total 8\n",
        "drwxr-xr-x 2 owner group 4096 2026-09-03 10:00:00 +0200 directory\n",
        "-rw-r----- 1 alice staff 123 2026-09-03 10:03:45.123456789 +0000 valid file.txt\n",
    );

    let entries = parse_ls_output(output).unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["name"], "directory");
    assert_eq!(entries[0]["size"], "0");
    assert_eq!(entries[0]["time"], "2026-09-03 10:00:00");
    assert_eq!(entries[1]["name"], "valid file.txt");
    assert_eq!(entries[1]["size"], "123");
    assert_eq!(entries[1]["time"], "2026-09-03 10:03:45.123456789");
    assert_eq!(entries[1]["owner"], "alice");
}

#[test]
fn test_ls_parse_skips_malformed_lines() {
    let output = concat!(
        "total 4\n",
        "not enough fields\n",
        "-rw-r----- 1 alice staff 123 2026-09-03 10:03 valid file.txt\n",
    );

    let entries = parse_ls_output(output).unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["owner"], "alice");
    assert_eq!(entries[0]["group"], "staff");
    assert_eq!(entries[0]["name"], "valid file.txt");
}

#[test]
fn test_ls_parse_rejects_error_output() {
    let output = concat!(
        "sudo: a terminal is required to read the password; either use the -S option ",
        "to read from standard input or configure an askpass helper\n",
        "sudo: a password is required\n",
    );

    let error = parse_ls_output(output).unwrap_err();
    assert!(error.contains("sudo:"));
}

#[test]
fn test_ls_parse_symlink_target_and_resolved_directories() {
    let output = concat!(
        "total 8\n",
        "drwxr-xr-x 2 root root 40 2025-05-22 01:17:00 +0000 real-dir\n",
        "lrwxrwxrwx 1 root root 8 2025-05-22 01:17:00 +0000 dir-link -> real-dir\n",
        "lrwxrwxrwx 1 root root 9 2025-05-22 01:17:00 +0000 file-link -> somewhere\n",
        "Resolved directories:\n",
        "dir-link\n",
    );

    let entries = parse_ls_output(output).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["name"], "real-dir");
    assert_eq!(entries[0]["type"], "d");
    assert!(entries[0].get("points_to_dir").is_none());
    assert_eq!(entries[1]["name"], "dir-link");
    assert_eq!(entries[1]["type"], "l");
    assert_eq!(entries[1]["link_target"], "real-dir");
    assert_eq!(entries[1]["points_to_dir"], true);
    assert_eq!(entries[2]["name"], "file-link");
    assert_eq!(entries[2]["link_target"], "somewhere");
    assert!(entries[2].get("points_to_dir").is_none());
}

#[test]
fn test_ls_parse_rejects_invalid_permissions_row() {
    let output = concat!(
        "total 4\n",
        "Administrator. It usually boils down to these three things:\n",
    );

    assert!(parse_ls_output(output).is_err());
}

#[test]
fn test_chmod_applies_ownership_first() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(
            r#"'sudo' 'chown' 'alice:staff' '/tmp/file' && 'sudo' 'chmod' '6750' '/tmp/file'"#,
            "",
            0,
        )
    };

    let module_id = FileBrowserChmod::get_metadata().module_spec.id.clone();
    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (FileBrowserChmod::get_metadata(), FileBrowserChmod::new_command_module),
    );

    harness.execute_command(
        &module_id,
        vec![
            "/tmp/file".to_string(),
            "6750".to_string(),
            "alice".to_string(),
            "staff".to_string(),
        ],
    );

    harness.verify_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
    });
}

#[test]
fn test_chmod_pads_numeric_mode() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(
            r#"'sudo' 'chmod' '00775' '/tmp/dir'"#,
            "",
            0,
        )
    };

    let module_id = FileBrowserChmod::get_metadata().module_spec.id.clone();
    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (FileBrowserChmod::get_metadata(), FileBrowserChmod::new_command_module),
    );

    harness.execute_command(
        &module_id,
        vec!["/tmp/dir".to_string(), "0775".to_string()],
    );

    harness.verify_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
    });
}

#[test]
fn test_chmod_keeps_three_digit_mode() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(
            r#"'sudo' 'chmod' '775' '/tmp/dir'"#,
            "",
            0,
        )
    };

    let module_id = FileBrowserChmod::get_metadata().module_spec.id.clone();
    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (FileBrowserChmod::get_metadata(), FileBrowserChmod::new_command_module),
    );

    harness.execute_command(
        &module_id,
        vec!["/tmp/dir".to_string(), "775".to_string()],
    );

    harness.verify_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
    });
}

#[test]
fn test_du_parse_output() {
    let output = concat!(
        "4\t/tmp/dir\n",
        "123\t/tmp/file with spaces\n",
        "du: cannot access '/missing': No such file or directory\n",
    );

    let sizes = parse_du_output(output);
    assert_eq!(sizes.len(), 2);
    assert_eq!(sizes[0]["path"], "/tmp/dir");
    assert_eq!(sizes[0]["size"], "4");
    assert_eq!(sizes[1]["path"], "/tmp/file with spaces");
    assert_eq!(sizes[1]["size"], "123");
}

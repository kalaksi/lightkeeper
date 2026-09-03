/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use lightkeeper::enums::Criticality;
use lightkeeper::module::*;
use lightkeeper::module::command::CommandModule;
use lightkeeper::module::command::internal::FileBrowserChmod;
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
fn test_chmod_applies_ownership_first() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(
            r#""sudo" "chown" "alice:staff" "/tmp/file" && "sudo" "chmod" "6750" "/tmp/file""#,
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
            r#""sudo" "chmod" "00775" "/tmp/dir""#,
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
            r#""sudo" "chmod" "775" "/tmp/dir""#,
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

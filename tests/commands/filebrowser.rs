/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use lightkeeper::module::command::internal::filebrowser::ls::parse_ls_output;


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

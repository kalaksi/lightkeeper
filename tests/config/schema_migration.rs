/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::fs;

use lightkeeper::configuration::{Configuration, MIGRATION_VERSION};

#[test]
fn configuration_read_upgrades_outdated_schema_and_persists() {
    let dir = crate::unique_target_dir("schema-migration");
    let dir_str = dir.to_string_lossy().to_string();
    fs::create_dir_all(&dir).unwrap();

    // Minimal outdated config: migrations should add groups introduced after version 1.
    fs::write(
        dir.join("config.yml"),
        "preferences:\n  show_charts: false\nschema_version: 1\n",
    )
    .unwrap();
    fs::write(dir.join("hosts.yml"), "hosts: {}\n").unwrap();
    fs::write(
        dir.join("groups.yml"),
        "groups:\n  defaults:\n    connectors:\n      ssh: {}\n",
    )
    .unwrap();

    let (main, _hosts, groups) = Configuration::read(&dir_str).unwrap();
    assert_eq!(main.schema_version, Some(MIGRATION_VERSION));
    assert!(main.preferences.show_charts);
    assert!(groups.groups.contains_key("nixos"));
    assert!(groups.groups.contains_key("host"));
    assert!(groups.groups.contains_key("storage"));
    assert!(groups.groups.contains_key("network"));
    assert!(groups.groups.contains_key("haproxy"));
    assert!(groups.groups.contains_key("podman"));

    let (main_reread, _hosts_reread, groups_reread) = Configuration::read(&dir_str).unwrap();
    assert_eq!(main_reread.schema_version, Some(MIGRATION_VERSION));
    assert!(main_reread.preferences.show_charts);
    assert!(groups_reread.groups.contains_key("host"));
    assert!(groups_reread.groups.contains_key("network"));
}

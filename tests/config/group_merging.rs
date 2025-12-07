/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::BTreeMap;
use lightkeeper::configuration::{Configuration, ConfigGroup, HostSettings, MonitorConfig, CommandConfig, ConnectorConfig, CustomCommandConfig};

#[test]
fn test_single_group_merging() {
    let mut groups = BTreeMap::new();
    
    let mut group1 = ConfigGroup::default();
    let mut monitor_config = MonitorConfig::default();
    monitor_config.settings.insert("key1".to_string(), "value1".to_string());
    group1.monitors.insert("monitor1".to_string(), monitor_config);
    
    groups.insert("group1".to_string(), group1);
    
    let mut host_settings = HostSettings::default();
    host_settings.groups.push("group1".to_string());
    
    let effective = Configuration::get_effective_group_config(&host_settings, &groups);
    
    assert!(effective.monitors.contains_key("monitor1"));
    assert_eq!(effective.monitors["monitor1"].settings["key1"], "value1");
}

#[test]
fn test_multiple_groups_merging() {
    let mut groups = BTreeMap::new();
    
    // First group
    let mut group1 = ConfigGroup::default();
    let mut monitor1_config = MonitorConfig::default();
    monitor1_config.settings.insert("key1".to_string(), "value1".to_string());
    group1.monitors.insert("monitor1".to_string(), monitor1_config);
    groups.insert("group1".to_string(), group1);
    
    // Second group
    let mut group2 = ConfigGroup::default();
    let mut monitor2_config = MonitorConfig::default();
    monitor2_config.settings.insert("key2".to_string(), "value2".to_string());
    group2.monitors.insert("monitor2".to_string(), monitor2_config);
    groups.insert("group2".to_string(), group2);
    
    let mut host_settings = HostSettings::default();
    host_settings.groups.push("group1".to_string());
    host_settings.groups.push("group2".to_string());
    
    let effective = Configuration::get_effective_group_config(&host_settings, &groups);
    
    assert!(effective.monitors.contains_key("monitor1"));
    assert!(effective.monitors.contains_key("monitor2"));
    assert_eq!(effective.monitors["monitor1"].settings["key1"], "value1");
    assert_eq!(effective.monitors["monitor2"].settings["key2"], "value2");
}

#[test]
fn test_group_precedence_later_group_wins() {
    let mut groups = BTreeMap::new();
    
    // First group
    let mut group1 = ConfigGroup::default();
    let mut monitor_config = MonitorConfig::default();
    monitor_config.settings.insert("key1".to_string(), "value1".to_string());
    monitor_config.is_critical = Some(false);
    group1.monitors.insert("monitor1".to_string(), monitor_config);
    groups.insert("group1".to_string(), group1);
    
    // Second group - overrides monitor1
    let mut group2 = ConfigGroup::default();
    let mut monitor_config2 = MonitorConfig::default();
    monitor_config2.settings.insert("key1".to_string(), "value2".to_string()); // Different value
    monitor_config2.is_critical = Some(true); // Different criticality
    group2.monitors.insert("monitor1".to_string(), monitor_config2);
    groups.insert("group2".to_string(), group2);
    
    let mut host_settings = HostSettings::default();
    host_settings.groups.push("group1".to_string());
    host_settings.groups.push("group2".to_string());
    
    let effective = Configuration::get_effective_group_config(&host_settings, &groups);
    
    assert_eq!(effective.monitors["monitor1"].is_critical, Some(true)); // is_critical from second group
    // Settings should be merged (extended), not replaced
    assert_eq!(effective.monitors["monitor1"].settings["key1"], "value2");
}

#[test]
fn test_host_overrides_take_precedence() {
    let mut groups = BTreeMap::new();
    
    // Group with monitor
    let mut group1 = ConfigGroup::default();
    let mut monitor_config = MonitorConfig::default();
    monitor_config.settings.insert("key1".to_string(), "group_value".to_string());
    group1.monitors.insert("monitor1".to_string(), monitor_config);
    groups.insert("group1".to_string(), group1);
    
    // Host override
    let mut host_settings = HostSettings::default();
    host_settings.groups.push("group1".to_string());
    
    let mut override_config = ConfigGroup::default();
    let mut monitor_override = MonitorConfig::default();
    monitor_override.settings.insert("key1".to_string(), "override_value".to_string());
    override_config.monitors.insert("monitor1".to_string(), monitor_override);
    host_settings.overrides = override_config;
    
    let effective = Configuration::get_effective_group_config(&host_settings, &groups);
    
    assert_eq!(effective.monitors["monitor1"].settings["key1"], "override_value");
}

#[test]
fn test_command_merging() {
    let mut groups = BTreeMap::new();
    
    // First group
    let mut group1 = ConfigGroup::default();
    let mut command_config = CommandConfig::default();
    command_config.settings.insert("key1".to_string(), "value1".to_string());
    group1.commands.insert("command1".to_string(), command_config);
    groups.insert("group1".to_string(), group1);
    
    // Second group
    let mut group2 = ConfigGroup::default();
    let mut command_config2 = CommandConfig::default();
    command_config2.settings.insert("key1".to_string(), "value2".to_string());
    command_config2.settings.insert("key2".to_string(), "value3".to_string());
    group2.commands.insert("command1".to_string(), command_config2);
    groups.insert("group2".to_string(), group2);
    
    let mut host_settings = HostSettings::default();
    host_settings.groups.push("group1".to_string());
    host_settings.groups.push("group2".to_string());
    
    let effective = Configuration::get_effective_group_config(&host_settings, &groups);
    
    assert_eq!(effective.commands["command1"].settings["key1"], "value2");
    assert_eq!(effective.commands["command1"].settings["key2"], "value3");
}

#[test]
fn test_connector_merging() {
    let mut groups = BTreeMap::new();
    
    // First group
    let mut group1 = ConfigGroup::default();
    let mut connector_config = ConnectorConfig::default();
    connector_config.settings.insert("key1".to_string(), "value1".to_string());
    group1.connectors.insert("connector1".to_string(), connector_config);
    groups.insert("group1".to_string(), group1);
    
    // Second group
    let mut group2 = ConfigGroup::default();
    let mut connector_config2 = ConnectorConfig::default();
    connector_config2.settings.insert("key1".to_string(), "value2".to_string());
    connector_config2.settings.insert("key2".to_string(), "value3".to_string());
    group2.connectors.insert("connector1".to_string(), connector_config2);
    groups.insert("group2".to_string(), group2);
    
    let mut host_settings = HostSettings::default();
    host_settings.groups.push("group1".to_string());
    host_settings.groups.push("group2".to_string());
    
    let effective = Configuration::get_effective_group_config(&host_settings, &groups);
    
    assert_eq!(effective.connectors["connector1"].settings["key1"], "value2");
    assert_eq!(effective.connectors["connector1"].settings["key2"], "value3");
}

#[test]
fn test_custom_commands_accumulate() {
    let mut groups = BTreeMap::new();
    
    // First group
    let mut group1 = ConfigGroup::default();
    group1.custom_commands.push(CustomCommandConfig {
        name: "custom1".to_string(),
        description: "First custom".to_string(),
        command: "echo 1".to_string(),
    });
    groups.insert("group1".to_string(), group1);
    
    // Second group
    let mut group2 = ConfigGroup::default();
    group2.custom_commands.push(CustomCommandConfig {
        name: "custom2".to_string(),
        description: "Second custom".to_string(),
        command: "echo 2".to_string(),
    });
    groups.insert("group2".to_string(), group2);
    
    let mut host_settings = HostSettings::default();
    host_settings.groups.push("group1".to_string());
    host_settings.groups.push("group2".to_string());
    
    let effective = Configuration::get_effective_group_config(&host_settings, &groups);
    
    // Custom commands should accumulate from all groups
    assert_eq!(effective.custom_commands.len(), 2);
    assert!(effective.custom_commands.iter().any(|c| c.name == "custom1"));
    assert!(effective.custom_commands.iter().any(|c| c.name == "custom2"));
}

#[test]
fn test_empty_groups() {
    let groups = BTreeMap::new();
    
    let host_settings = HostSettings::default();
    
    let effective = Configuration::get_effective_group_config(&host_settings, &groups);
    
    assert!(effective.monitors.is_empty());
    assert!(effective.commands.is_empty());
    assert!(effective.connectors.is_empty());
    assert!(effective.custom_commands.is_empty());
}

#[test]
fn test_nonexistent_group_ignored() {
    let mut groups = BTreeMap::new();
    
    let mut group1 = ConfigGroup::default();
    let mut monitor_config = MonitorConfig::default();
    monitor_config.settings.insert("key1".to_string(), "value1".to_string());
    group1.monitors.insert("monitor1".to_string(), monitor_config);
    groups.insert("group1".to_string(), group1);
    
    let mut host_settings = HostSettings::default();
    host_settings.groups.push("group1".to_string());
    host_settings.groups.push("nonexistent_group".to_string());
    
    let effective = Configuration::get_effective_group_config(&host_settings, &groups);
    
    // Should still work with existing group
    assert!(effective.monitors.contains_key("monitor1"));
    assert_eq!(effective.monitors["monitor1"].settings["key1"], "value1");
}

#[test]
fn test_monitor_enabled_flag_merging() {
    let mut groups = BTreeMap::new();
    
    // First group - monitor enabled
    let mut group1 = ConfigGroup::default();
    let mut monitor_config = MonitorConfig::default();
    monitor_config.enabled = Some(true);
    group1.monitors.insert("monitor1".to_string(), monitor_config);
    groups.insert("group1".to_string(), group1);
    
    // Second group - disables monitor
    let mut group2 = ConfigGroup::default();
    let mut monitor_config2 = MonitorConfig::default();
    monitor_config2.enabled = Some(false);
    group2.monitors.insert("monitor1".to_string(), monitor_config2);
    groups.insert("group2".to_string(), group2);
    
    let mut host_settings = HostSettings::default();
    host_settings.groups.push("group1".to_string());
    host_settings.groups.push("group2".to_string());
    
    let effective = Configuration::get_effective_group_config(&host_settings, &groups);
    
    // Later group should win
    assert_eq!(effective.monitors["monitor1"].enabled, Some(false));
}
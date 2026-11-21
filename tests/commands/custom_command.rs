/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::collections::{BTreeMap, HashMap};

use lightkeeper::configuration;
use lightkeeper::enums::Criticality;
use lightkeeper::module::command::CommandModule;
use lightkeeper::module::command::internal::CustomCommand;
use lightkeeper::module::connection::Connector;
use lightkeeper::module::platform_info::*;
use lightkeeper::module::*;
use lightkeeper::HostSetting;
use lightkeeper::ModuleFactory;

use crate::{CommandTestHarness, StubSsh2, TEST_HOST_ID};

const MODULE_ID: &str = "_custom-command";
const COMMAND_NAME: &str = "say-hello";

fn harness_for_custom_command(
    command_text: &str,
    new_stub_ssh: fn(&HashMap<String, String>) -> Connector,
) -> CommandTestHarness {
    let mut host_settings = configuration::HostSettings::default();
    host_settings.address = "127.0.0.1".to_string();
    host_settings.effective.host_settings = vec![HostSetting::UseSudo];
    host_settings.effective.custom_commands.push(configuration::CustomCommandConfig {
        name: COMMAND_NAME.to_string(),
        description: String::new(),
        command: command_text.to_string(),
    });
    // So ConnectionManager creates the ssh connector (it only walks effective.commands/monitors).
    host_settings.effective.commands.insert(
        MODULE_ID.to_string(),
        configuration::CommandConfig {
            version: "0.0.1".to_string(),
            ..Default::default()
        },
    );
    host_settings.effective.connectors.insert(
        StubSsh2::get_metadata().module_spec.id.clone(),
        configuration::ConnectorConfig::default(),
    );

    let hosts_config = configuration::Hosts {
        hosts: BTreeMap::from([(TEST_HOST_ID.to_string(), host_settings)]),
        predefined_platforms: BTreeMap::from([(
            TEST_HOST_ID.to_string(),
            PlatformInfo::linux(Flavor::Debian, "12.0"),
        )]),
        ..Default::default()
    };

    // CustomCommand is also registered as an internal command in CommandHandler::configure.
    let module_factory = ModuleFactory::new_with(
        vec![(StubSsh2::get_metadata(), new_stub_ssh)],
        vec![],
        vec![(CustomCommand::get_metadata(), CustomCommand::new_command_module)],
    );
    CommandTestHarness::new(hosts_config, module_factory)
}

#[test]
fn test_custom_command_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#"'sh' '-c' 'echo hello'"#, "hello", 0)
    };

    let mut harness = harness_for_custom_command("echo hello", new_stub_ssh);

    harness.execute_command(MODULE_ID, vec![COMMAND_NAME.to_string()]);

    harness.verify_next_command_result(MODULE_ID, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
        assert_eq!(result.progress, 100);
        assert!(result.message.contains("hello"));
    });
}

#[test]
fn test_custom_command_nonzero() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#"'sh' '-c' 'false'"#, "command failed", 1)
    };

    let mut harness = harness_for_custom_command("false", new_stub_ssh);

    harness.execute_command(MODULE_ID, vec![COMMAND_NAME.to_string()]);

    harness.verify_next_command_result(MODULE_ID, |result| {
        assert_eq!(result.criticality, Criticality::Error);
        assert_eq!(result.progress, 100);
        assert!(result.message.contains("command failed"));
    });
}

#[test]
fn test_custom_command_partial_streaming() {
    // Longer than StubSsh2's 20-byte partial chunk size so FollowOutput yields multiple updates.
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(
            r#"'sh' '-c' 'echo streaming'"#,
            "line one of custom output\nline two of custom output\nline three done",
            0,
        )
    };

    let mut harness = harness_for_custom_command("echo streaming", new_stub_ssh);

    harness.execute_command(MODULE_ID, vec![COMMAND_NAME.to_string()]);

    harness.verify_next_ui_update(|display_data| {
        assert_eq!(display_data.host_state.command_invocations.len(), 1);
        let result = CommandTestHarness::expect_new_command_result(display_data, MODULE_ID);
        assert!(result.progress < 100);
        assert!(!result.message.is_empty());
    });

    let result = harness.recv_command_result_until(MODULE_ID, |result| result.progress == 100);
    assert_eq!(result.criticality, Criticality::Normal);
    assert!(result.message.contains("line three"));
}

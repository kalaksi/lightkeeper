/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::{BTreeMap, HashMap, HashSet};

use lightkeeper::configuration;
use lightkeeper::module::*;
use lightkeeper::module::command::systemd;
use lightkeeper::module::command::CommandModule;
use lightkeeper::module::monitoring;
use lightkeeper::module::monitoring::MonitoringModule;
use lightkeeper::module::platform_info::*;
use lightkeeper::HostSetting;

use crate::{MonitorTestHarness, StubSsh2, TEST_HOST_ID};

#[test]
fn invocation_ids_are_globally_unique() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| StubSsh2::new_any("", 0);

    let mut host_settings = configuration::HostSettings::default();
    host_settings.address = "127.0.0.1".to_string();
    host_settings.effective.host_settings = vec![HostSetting::UseSudo];
    host_settings.effective.commands.insert(
        systemd::service::Start::get_metadata().module_spec.id.clone(),
        configuration::CommandConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        },
    );
    host_settings.effective.connectors.insert(
        StubSsh2::get_metadata().module_spec.id.clone(),
        configuration::ConnectorConfig::default(),
    );
    host_settings.effective.monitors.insert(
        monitoring::os::Os::get_metadata().module_spec.id.clone(),
        configuration::MonitorConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        },
    );

    let hosts_config = configuration::Hosts {
        hosts: BTreeMap::from([(TEST_HOST_ID.to_string(), host_settings)]),
        predefined_platforms: BTreeMap::from([(
            TEST_HOST_ID.to_string(),
            PlatformInfo::linux(Flavor::Debian, "12.0"),
        )]),
        ..Default::default()
    };

    let module_factory = ModuleFactory::new_with(
        vec![(StubSsh2::get_metadata(), new_stub_ssh)],
        vec![(monitoring::os::Os::get_metadata(), monitoring::os::Os::new_monitoring_module)],
        vec![(systemd::service::Start::get_metadata(), systemd::service::Start::new_command_module)],
    );

    let mut harness = MonitorTestHarness::new_with_command_handler(hosts_config, module_factory);

    let mut ids: Vec<u64> = Vec::new();

    let command_handler = harness.command_handler.as_mut().unwrap();
    let command_id = systemd::service::Start::get_metadata().module_spec.id.clone();
    ids.push(command_handler.execute(TEST_HOST_ID, &command_id, &["test-service.service".to_string()]));
    ids.push(command_handler.execute(TEST_HOST_ID, &command_id, &["other.service".to_string()]));
    ids.push(command_handler.execute(TEST_HOST_ID, &command_id, &["third.service".to_string()]));

    for category in harness.monitor_manager.get_all_host_categories(TEST_HOST_ID) {
        ids.extend(
            harness
                .monitor_manager
                .refresh_monitors_of_category(TEST_HOST_ID, &category),
        );
    }

    let unique_count = ids.iter().copied().collect::<HashSet<_>>().len();
    assert_eq!(ids.len(), unique_count, "invocation IDs must be globally unique");
}

/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::{BTreeMap, HashMap};

use lightkeeper::module::*;
use lightkeeper::module::monitoring::*;
use lightkeeper::module::monitoring::linux;
use lightkeeper::module::platform_info::*;
use lightkeeper::configuration;

use crate::{MonitorTestHarness, StubSsh2, TEST_HOST_ID};


#[test]
fn test_platform_info_ssh_refresh() {
    // Platform info is automatically refreshed when monitors using SSH connector are configured
    // This test verifies that platform info refresh works correctly
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        let mut ssh = StubSsh2::default();
        // Platform info requires two commands: cat /etc/os-release and uname -m
        // TODO: auto-generated, check or replace.
        ssh.add_response("cat /etc/os-release",
r#"ID=debian
VERSION_ID="12.0"
PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
"#, 0);
        // TODO: auto-generated, check or replace.
        ssh.add_response("uname -m", "x86_64", 0);
        // TODO: auto-generated, check or replace.
        ssh.add_response("uname -r -m", "6.1.0-41-amd64 x86_64", 0);
        
        Box::new(ssh) as connection::Connector
    };

    let mut host_settings = configuration::HostSettings::default();
    host_settings.address = "127.0.0.1".to_string();
    host_settings.effective.monitors.insert(
        linux::Kernel::get_metadata().module_spec.id.clone(),
        configuration::MonitorConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        }
    );
    host_settings.effective.connectors.insert(
        StubSsh2::get_metadata().module_spec.id.clone(),
        configuration::ConnectorConfig::default()
    );

    let hosts_config = configuration::Hosts {
        hosts: BTreeMap::from([(TEST_HOST_ID.to_string(), host_settings)]),
        predefined_platforms: BTreeMap::new(), // Start with unset platform
        ..Default::default()
    };

    let module_factory = ModuleFactory::new_with(
        vec![(StubSsh2::get_metadata(), new_stub_ssh)],
        vec![(linux::Kernel::get_metadata(), linux::Kernel::new_monitoring_module)],
        vec![]
    );

    let mut harness = MonitorTestHarness::new(hosts_config, module_factory);

    // Manually trigger platform info refresh
    harness.monitor_manager.refresh_platform_info(TEST_HOST_ID);
    
    // Wait for platform info to be set by polling with timeout
    let mut attempts = 0;
    loop {
        harness.wait_for_completion();
        let display_data = harness.host_manager.borrow().get_display_data();
        if let Some(host_display) = display_data.hosts.get(TEST_HOST_ID) {
            let platform = &host_display.host_state.host.platform;
            if platform.is_set() {
                assert!(matches!(platform.os, OperatingSystem::Linux));
                assert!(matches!(platform.os_flavor, Flavor::Debian));
                break;
            }
        }
        attempts += 1;
        if attempts > 20 {
            panic!("Platform info was not set within timeout");
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

#[test]
fn test_platform_info_refresh_all() {
    // Test refresh_platform_info_all functionality
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        let mut ssh = StubSsh2::default();
        // TODO: auto-generated, check or replace.
        ssh.add_response("cat /etc/os-release",
r#"ID=ubuntu
VERSION_ID="22.04"
PRETTY_NAME="Ubuntu 22.04 LTS"
"#, 0);
        // TODO: auto-generated, check or replace.
        ssh.add_response("uname -m", "arm64", 0);
        // TODO: auto-generated, check or replace.
        ssh.add_response("uname -r -m", "6.1.0-41-amd64 aarch64", 0);
        
        Box::new(ssh) as connection::Connector
    };

    let mut host_settings = configuration::HostSettings::default();
    host_settings.address = "127.0.0.1".to_string();
    host_settings.effective.monitors.insert(
        linux::Kernel::get_metadata().module_spec.id.clone(),
        configuration::MonitorConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        }
    );
    host_settings.effective.connectors.insert(
        StubSsh2::get_metadata().module_spec.id.clone(),
        configuration::ConnectorConfig::default()
    );

    let hosts_config = configuration::Hosts {
        hosts: BTreeMap::from([(TEST_HOST_ID.to_string(), host_settings)]),
        predefined_platforms: BTreeMap::new(), // No predefined platform
        ..Default::default()
    };

    let module_factory = ModuleFactory::new_with(
        vec![(StubSsh2::get_metadata(), new_stub_ssh)],
        vec![(linux::Kernel::get_metadata(), linux::Kernel::new_monitoring_module)],
        vec![]
    );

    let mut harness = MonitorTestHarness::new(hosts_config, module_factory);
    
    // Manually trigger platform info refresh for all hosts
    harness.monitor_manager.refresh_platform_info_all();
    
    // Wait for platform info to be set by polling with timeout
    let mut attempts = 0;
    loop {
        harness.wait_for_completion();
        let display_data = harness.host_manager.borrow().get_display_data();
        if let Some(host_display) = display_data.hosts.get(TEST_HOST_ID) {
            let platform = &host_display.host_state.host.platform;
            if platform.is_set() {
                assert!(matches!(platform.os_flavor, Flavor::Ubuntu));
                assert!(matches!(platform.architecture, Architecture::Arm64));
                break;
            }
        }
        attempts += 1;
        if attempts > 20 {
            panic!("Platform info was not set within timeout");
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}


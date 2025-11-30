/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

mod common;
pub use common::*;
use lightkeeper::module::connection::{ConnectionModule, ResponseMessage};
use lightkeeper::module::monitoring::MonitoringModule;
use lightkeeper::module::platform_info::*;
use lightkeeper::module::*;

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;


use lightkeeper::*;

// Import internal types for testing (modules are public in test mode)
use lightkeeper::host_manager::HostManager;
use lightkeeper::connection_manager::ConnectionManager;
use lightkeeper::command_handler::CommandHandler;
use lightkeeper::monitor_manager::MonitorManager;

// Test harness for monitoring module tests
struct TestHarness {
    host_manager: Rc<RefCell<HostManager>>,
    connection_manager: ConnectionManager,
    monitor_manager: MonitorManager,
}

impl TestHarness {
    fn new(
        main_config: Configuration,
        hosts_config: configuration::Hosts,
        module_factory: ModuleFactory,
    ) -> TestHarness {

        let _ = env_logger::Builder::from_default_env()
            .is_test(true)
            .try_init();

        let module_factory = Arc::<ModuleFactory>::new(module_factory);

        let host_manager = Rc::new(RefCell::new(HostManager::new()));
        host_manager.borrow_mut().configure(&hosts_config);

        let mut connection_manager = ConnectionManager::new(module_factory.clone());
        connection_manager.configure(&hosts_config);

        let mut monitor_manager = MonitorManager::new(host_manager.clone(), module_factory.clone());
        monitor_manager.configure(
            &hosts_config,
            connection_manager.new_request_sender(),
            host_manager.borrow().new_state_update_sender()
        );

        let mut command_handler = CommandHandler::new(host_manager.clone(), module_factory.clone());
        command_handler.configure(
            &hosts_config,
            &main_config.preferences,
            connection_manager.new_request_sender(),
            host_manager.borrow().new_state_update_sender()
        );

        // Start backend threads.
        host_manager.borrow_mut().start_receiving_updates();
        connection_manager.start_processing_requests();
        command_handler.start_processing_responses();
        monitor_manager.start_processing_responses();

        // TODO: Needs a proper fix for a race.
        // Wait a small amount as a workaround so initial data points have time to get sent to host manager.
        // Otherwise, initial status summary icons are randomly not shown.
        std::thread::sleep(std::time::Duration::from_millis(100));

        TestHarness {
            host_manager,
            connection_manager,
            monitor_manager,
        }
    }

    fn new_monitor_tester(
        connector_module: (Metadata, fn(&HashMap<String, String>) -> connection::Connector),
        monitor_module: (Metadata, fn(&HashMap<String, String>) -> monitoring::Monitor),
    ) -> TestHarness {

        // Test host
        let mut host_settings = configuration::HostSettings::default();

        host_settings.address = "127.0.0.1".to_string();

        host_settings.effective.monitors.insert(
            monitor_module.0.module_spec.id.clone(),
            configuration::MonitorConfig {
                version: "0.0.1".to_string(),
                settings: HashMap::new(),
                ..Default::default()
            }
        );
        

        host_settings.effective.connectors.insert(
            "ssh".to_string(),
            configuration::ConnectorConfig::default()
        );
        
        let hosts_config = configuration::Hosts {
            hosts: BTreeMap::from([
                ("test-host".to_string(), host_settings)
            ]),
            ..Default::default()
        };

        let module_factory = ModuleFactory::new_with(
            vec![ connector_module ],
            vec![ monitor_module ],
            vec![]
        );

        let main_config = configuration::Configuration::default();

        TestHarness::new(main_config, hosts_config, module_factory)
    }

    fn refresh_monitor(&mut self, monitor_id: &str) -> Vec<u64> {
        let result = self.monitor_manager.refresh_monitors_by_id(&"test-host".to_string(), &monitor_id.to_string());
        assert!(!result.is_empty(), "Monitor refresh should return an invocation ID");
    }

    fn wait_for_completion(&self) {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    fn verify_monitor_data<F>(&self, monitor_id: &str, verify_fn: F)
    where
        F: FnOnce(&lightkeeper::module::monitoring::MonitoringData),
    {
        let display_data = self.host_manager.borrow().get_display_data();
        let host_display = display_data.hosts.get("test-host");
        
        assert!(host_display.is_some(), "Host should exist in display data");
        
        let host_state = &host_display.unwrap().host_state;
        
        assert!(
            host_state.monitor_data.contains_key(monitor_id),
            "{} monitor data should be in host state",
            monitor_id
        );
        
        let monitor_data = host_state.monitor_data.get(monitor_id).unwrap();
        assert!(
            !monitor_data.values.is_empty(),
            "{} monitor should have at least one data point",
            monitor_id
        );
        
        verify_fn(monitor_data);
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        self.monitor_manager.stop();
        self.connection_manager.stop();
        self.host_manager.borrow_mut().stop();
    }
}


#[test]
fn test_uptime() {
    // Customize SSH stub module creation.
    let create_stub_ssh = |_settings: &HashMap<String, String>| {
        let mut ssh = StubSsh2::new(Flavor::Debian);
        ssh.add_response("uptime", ResponseMessage::new_success(" 17:26:40 up 16 days,  4:25,  1 user,  load average: 0.06, 0.05, 0.01"));

        Box::new(ssh) as connection::Connector
    };

    let mut harness = TestHarness::new_monitor_tester(
        (StubSsh2::get_metadata(), create_stub_ssh),
        (monitoring::linux::Uptime::get_metadata(), monitoring::linux::Uptime::new_monitoring_module),
    );

    let invocation_ids = harness.refresh_monitor("uptime");

    harness.wait_for_completion();
    harness.verify_monitor_data("uptime", |uptime_data| {
        let latest_datapoint = uptime_data.values.back().unwrap();
        assert_eq!(latest_datapoint.value, "16", "Uptime should be parsed as 16 days");
    });
}

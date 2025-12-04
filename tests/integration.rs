/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

mod common;
mod monitors;

pub use common::*;
use lightkeeper::module::monitoring::*;
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
use lightkeeper::monitor_manager::MonitorManager;

const TEST_HOST_ID: &str = "test-host";

// Test harness for monitoring module tests
struct MonitorTestHarness {
    host_manager: Rc<RefCell<HostManager>>,
    connection_manager: ConnectionManager,
    monitor_manager: MonitorManager,
}

impl MonitorTestHarness {
    fn new(hosts_config: configuration::Hosts, module_factory: ModuleFactory) -> MonitorTestHarness {
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

        // Start backend threads.
        host_manager.borrow_mut().start_receiving_updates();
        connection_manager.start_processing_requests();
        monitor_manager.start_processing_responses();

        // TODO: Needs a proper fix for a race.
        // Wait a small amount as a workaround so initial data points have time to get sent to host manager.
        // Otherwise, initial status summary icons are randomly not shown.
        std::thread::sleep(std::time::Duration::from_millis(100));

        MonitorTestHarness {
            host_manager,
            connection_manager,
            monitor_manager,
        }
    }

    fn new_monitor_tester(
        platform_info: PlatformInfo,
        connector_module: (Metadata, fn(&HashMap<String, String>) -> connection::Connector),
        monitor_module: (Metadata, fn(&HashMap<String, String>) -> monitoring::Monitor),
    ) -> MonitorTestHarness {

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
            connector_module.0.module_spec.id.clone(),
            configuration::ConnectorConfig::default()
        );
        
        let hosts_config = configuration::Hosts {
            hosts: BTreeMap::from([(TEST_HOST_ID.to_string(), host_settings)]),
            predefined_platforms: BTreeMap::from([(TEST_HOST_ID.to_string(), platform_info)]),
            ..Default::default()
        };

        let module_factory = ModuleFactory::new_with(
            vec![ connector_module ],
            vec![ monitor_module ],
            vec![]
        );

        MonitorTestHarness::new(hosts_config, module_factory)
    }

    fn refresh_monitors(&mut self) {
        for category in self.monitor_manager.get_all_host_categories(TEST_HOST_ID) {
            let _invocation_ids = self.monitor_manager.refresh_monitors_of_category(TEST_HOST_ID, &category);
        }

        self.wait_for_completion();
    }

    fn wait_for_completion(&self) {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    fn verify_monitor_data<F>(&self, monitor_id: &str, verify_fn: F)
    where
        F: FnOnce(&DataPoint),
    {
        let display_data = self.host_manager.borrow().get_display_data();
        let host_display = display_data.hosts.get(TEST_HOST_ID);
        
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
        
        let latest_datapoint = monitor_data.values.back().unwrap();

        verify_fn(latest_datapoint);
    }
}

impl Drop for MonitorTestHarness {
    fn drop(&mut self) {
        self.monitor_manager.stop();
        self.connection_manager.stop();
        self.host_manager.borrow_mut().stop();
    }
}
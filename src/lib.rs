/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

extern crate openssl;

pub mod error;
pub mod module;
pub mod configuration;
mod host_manager;
mod monitor_manager;
mod host;
mod utils;
mod enums;
mod frontend;
mod connection_manager;
mod command_handler;
mod file_handler;
mod metrics;

pub use module::ModuleFactory;
pub use configuration::Configuration;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use host_manager::HostManager;
use monitor_manager::MonitorManager;
use connection_manager::ConnectionManager;
use command_handler::CommandHandler;
use host::Host;


#[derive(PartialEq)]
pub enum ExitReason {
    Quit,
    Restart,
}

pub fn run(
    config_dir: &String,
    main_config: &Configuration,
    hosts_config: &configuration::Hosts,
    group_config: &configuration::Groups,
    test: bool) -> Result<ExitReason, String> {

    // Due to some weirdness between ssh2, openssl and qmetaobject crates, openssl initialization needs to be forced here.
    // Otherwise, there might be a problem in ssh2 handshake with error "Unable to exchange encryption keys" for no apparent reason.
    let _ = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls())
        .map_err(|error| format!("Failed to initialize OpenSSL: {}", error))?;

    let module_factory = Arc::<ModuleFactory>::new(ModuleFactory::new());
    let module_metadatas = module_factory.get_module_metadatas();
    let mut frontend = frontend::qt::QmlFrontend::new(
        config_dir,
        main_config,
        hosts_config,
        group_config,
        module_metadatas,
    );

    let host_manager = Rc::new(RefCell::new(HostManager::new()));
    host_manager.borrow_mut().configure(&hosts_config);

    let mut connection_manager = ConnectionManager::new(module_factory.clone());
    connection_manager.configure(&hosts_config);

    let metrics_manager = if main_config.preferences.show_charts {
        Some(metrics::MetricsManager::new(frontend.new_update_sender()))
    }
    else {
        log::debug!("Charts are disabled.");
        None
    };

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

    host_manager.borrow_mut().add_observer(frontend.new_update_sender());
    if !test {
        Ok(frontend.start(command_handler, monitor_manager, connection_manager, host_manager, metrics_manager))
    }
    else {
        // TODO
        Ok(ExitReason::Quit)
    }
}
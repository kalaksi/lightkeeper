/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

extern crate openssl;

pub mod error;
pub mod module;
pub mod configuration;
// Made public for integration tests
pub mod host_manager;
pub mod connection_manager;
pub mod command_handler;
pub mod monitor_manager;
pub mod secrets_manager;
mod host;
pub use host::HostSetting;
pub mod utils;
pub mod enums;
// Made public for integration tests
pub mod frontend;
pub mod file_handler;
mod metrics;
pub mod remote_core;
pub mod backend;

pub use module::ModuleFactory;
pub use configuration::Configuration;
use std::sync::atomic::AtomicU64;
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

pub struct CoreComponents {
    pub module_factory: Arc<ModuleFactory>,
    pub host_manager: Rc<RefCell<HostManager>>,
    pub connection_manager: ConnectionManager,
    pub command_handler: CommandHandler,
    pub monitor_manager: MonitorManager,
}

pub fn initialize_openssl() -> Result<(), error::LkError> {
    // Due to some weirdness between ssh2, openssl and qmetaobject crates, openssl initialization needs to be forced here.
    // Otherwise, there might be a problem in ssh2 handshake with error "Unable to exchange encryption keys" for no apparent reason.
    let _ = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls())
        .map_err(|error| error::LkError::other(format!("Failed to initialize OpenSSL: {}", error)))?;

    Ok(())
}

pub fn initialize_core_with_module_factory(
    main_config: &Configuration,
    hosts_config: &configuration::Hosts,
    module_factory: Arc<ModuleFactory>,
) -> Result<CoreComponents, error::LkError> {
    initialize_openssl()?;

    let host_manager = Rc::new(RefCell::new(HostManager::new()));
    host_manager.borrow_mut().configure(hosts_config);

    let mut connection_manager = ConnectionManager::new(module_factory.clone());
    connection_manager.configure(hosts_config);

    let invocation_id_counter = Arc::new(AtomicU64::new(0));

    let mut monitor_manager = MonitorManager::new(
        host_manager.clone(),
        module_factory.clone(),
        invocation_id_counter.clone(),
    );
    monitor_manager.configure(
        hosts_config,
        connection_manager.new_request_sender(),
        host_manager.borrow().new_state_update_sender(),
    );

    let mut command_handler = CommandHandler::new(
        host_manager.clone(),
        module_factory.clone(),
        invocation_id_counter,
    );
    command_handler.configure(
        hosts_config,
        &main_config.preferences,
        connection_manager.new_request_sender(),
        host_manager.borrow().new_state_update_sender(),
    );

    host_manager.borrow_mut().start_receiving_updates();
    connection_manager.start_processing_requests();
    command_handler.start_processing_responses();
    monitor_manager.start_processing_responses();

    Ok(CoreComponents {
        module_factory,
        host_manager,
        connection_manager,
        command_handler,
        monitor_manager,
    })
}

pub fn initialize_core(
    main_config: &Configuration,
    hosts_config: &configuration::Hosts,
) -> Result<CoreComponents, error::LkError> {
    initialize_core_with_module_factory(
        main_config,
        hosts_config,
        Arc::new(ModuleFactory::new()),
    )
}

pub fn run(
    config_dir: &String,
    main_config: &Configuration,
    hosts_config: &configuration::Hosts,
    group_config: &configuration::Groups,
    test: bool,
) -> Result<ExitReason, String> {
    let CoreComponents {
        module_factory,
        host_manager,
        connection_manager,
        command_handler,
        monitor_manager,
    } = initialize_core(main_config, hosts_config).map_err(String::from)?;

    let module_metadatas = module_factory.get_module_metadatas();
    let command_backend = backend::new_local_command_backend(command_handler, monitor_manager);

    let mut frontend = frontend::qt::QmlFrontend::new(
        config_dir,
        main_config,
        hosts_config,
        group_config,
        module_metadatas,
    );

    let metrics_manager = if main_config.preferences.show_charts {
        Some(metrics::MetricsManager::new(frontend.new_update_sender()))
    }
    else {
        log::debug!("Charts are disabled.");
        None
    };

    // TODO: Needs a proper fix for a race.
    // Wait a small amount as a workaround so initial data points have time to get sent to host manager.
    // Otherwise, initial status summary icons are randomly not shown.
    std::thread::sleep(std::time::Duration::from_millis(100));

    host_manager.borrow_mut().add_observer(frontend.new_update_sender());
    if !test {
        Ok(frontend.start(command_backend, connection_manager, host_manager, metrics_manager, false))
    }
    else {
        // TODO
        Ok(ExitReason::Quit)
    }
}
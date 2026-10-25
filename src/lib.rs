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
pub mod metrics;
pub mod file_handler;
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
    /// Restart without automatically connecting to remote core.
    RestartLocal,
}

pub struct CoreComponents {
    pub module_factory: Arc<ModuleFactory>,
    pub host_manager: Rc<RefCell<HostManager>>,
    pub connection_manager: ConnectionManager,
    pub command_handler: CommandHandler,
    pub monitor_manager: MonitorManager,
}

impl CoreComponents {
    /// Reconfigure managers in place, retaining host states for unchanged hosts (same as local reload).
    /// Returns host IDs whose state was reset due to a configuration change.
    pub fn reconfigure(
        &mut self,
        main_config: &Configuration,
        hosts_config: &configuration::Hosts,
        secret_store: Arc<dyn secrets_manager::SecretStore>,
    ) -> Vec<String> {
        let mut hosts_config = hosts_config.clone();
        Configuration::resolve_secrets_in_hosts(&mut hosts_config, secret_store);

        self.connection_manager.configure(&hosts_config);
        let reset_hosts = self.host_manager.borrow_mut().configure(&hosts_config);
        self.monitor_manager.configure(
            &hosts_config,
            self.connection_manager.new_request_sender(),
            self.host_manager.borrow().new_state_update_sender(),
        );
        self.command_handler.configure(
            &hosts_config,
            &main_config.preferences,
            self.connection_manager.new_request_sender(),
            self.host_manager.borrow().new_state_update_sender(),
        );

        self.host_manager.borrow_mut().start_receiving_updates();
        self.connection_manager.start_processing_requests();
        self.command_handler.start_processing_responses();
        self.monitor_manager.start_processing_responses();
        self.monitor_manager.start_auto_refresh(
            &hosts_config,
            main_config.preferences.auto_refresh_interval,
        );

        reset_hosts
    }
}

pub fn initialize_openssl() -> Result<(), error::LkError> {
    // Due to some weirdness between ssh2, openssl and qmetaobject crates, openssl initialization needs to be forced here.
    // Otherwise, there might be a problem in ssh2 handshake with error "Unable to exchange encryption keys" for no apparent reason.
    let _ = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls())
        .map_err(|error| error::LkError::other(format!("Failed to initialize OpenSSL: {}", error)))?;

    Ok(())
}

pub fn initialize_core(
    main_config: &Configuration,
    hosts_config: &configuration::Hosts,
    module_factory: Arc<ModuleFactory>,
    secret_store: Arc<dyn secrets_manager::SecretStore>,
) -> Result<CoreComponents, error::LkError> {

    initialize_openssl()?;

    if let Err(error) = file_handler::remove_expired_temporary_files() {
        log::error!("Failed to clean up expired temporary cache files: {}", error);
    }

    let host_manager = Rc::new(RefCell::new(HostManager::new()));
    let connection_manager = ConnectionManager::new(module_factory.clone());
    let invocation_id_counter = Arc::new(AtomicU64::new(0));
    let monitor_manager = MonitorManager::new(
        host_manager.clone(),
        module_factory.clone(),
        invocation_id_counter.clone(),
    );
    let command_handler = CommandHandler::new(
        host_manager.clone(),
        module_factory.clone(),
        invocation_id_counter,
    );

    let mut core = CoreComponents {
        module_factory,
        host_manager,
        connection_manager,
        command_handler,
        monitor_manager,
    };
    core.reconfigure(main_config, hosts_config, secret_store);

    Ok(core)
}

#[cfg(feature = "gui")]
pub fn run(
    config_dir: &String,
    main_config: &Configuration,
    hosts_config: &configuration::Hosts,
    group_config: &configuration::Groups,
    skip_core_auto_connect: bool,
    test: bool,
) -> Result<ExitReason, String> {

    let secret_store: Arc<dyn secrets_manager::SecretStore> = Arc::new(secrets_manager::KeyringSecretStore);
    let CoreComponents {
        module_factory,
        host_manager,
        connection_manager,
        command_handler,
        monitor_manager,
    } = initialize_core(
        main_config,
        hosts_config,
        Arc::new(ModuleFactory::new()),
        secret_store.clone(),
    ).map_err(String::from)?;

    let module_metadatas = module_factory.get_module_metadatas();
    let command_backend: Box<dyn backend::CommandBackend> =
        Box::new(backend::LocalCommandBackend::new(command_handler, monitor_manager));
    let config_backend: Box<dyn backend::ConfigBackend> =
        Box::new(backend::LocalConfigBackend::new(config_dir.clone(), secret_store));

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

    // Observer has to be added before the frontend takes its initial display data in start(). Otherwise, initial
    // data points processed in between are lost and initial status summary icons are randomly not shown.
    host_manager.borrow_mut().add_observer(frontend.new_update_sender());
    if !test {
        Ok(frontend.start(
            command_backend,
            config_backend,
            connection_manager,
            host_manager,
            metrics_manager,
            false,
            skip_core_auto_connect,
        ))
    }
    else {
        // TODO
        Ok(ExitReason::Quit)
    }
}
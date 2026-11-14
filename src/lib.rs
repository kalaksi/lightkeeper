/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
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
        start_connection_processing: bool,
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
        if start_connection_processing {
            self.connection_manager.start_processing_requests();
        }
        self.command_handler.start_processing_responses();
        self.monitor_manager.start_processing_responses();
        self.monitor_manager.start_timers(
            &hosts_config,
            main_config.preferences.auto_refresh_interval,
            main_config.preferences.cert_monitor_refresh_interval,
            self.host_manager.borrow().frontend_update_senders(),
        );

        reset_hosts
    }
}

/// Applies config updates that running managers can take without reconfigure(), like local mode soft writes.
/// Returns false if reconfigure() is still needed.
pub fn apply_soft_config_update(
    host_manager: &HostManager,
    monitor_manager: &mut MonitorManager,
    secret_store: Arc<dyn secrets_manager::SecretStore>,
    previous_main_config: &Configuration,
    previous_hosts_config: &configuration::Hosts,
    main_config: &Configuration,
    hosts_config: &configuration::Hosts,
) -> bool {
    // Only used by the frontend or on startup. show_charts is handled separately by the metrics owner.
    // Timer intervals are applied below via start_timers without a full reconfigure.
    let previous_preferences = &previous_main_config.preferences;
    let mut preferences = main_config.preferences.clone();
    preferences.refresh_hosts_on_start = previous_preferences.refresh_hosts_on_start;
    preferences.close_to_tray = previous_preferences.close_to_tray;
    preferences.show_monitor_notifications = previous_preferences.show_monitor_notifications;
    preferences.show_charts = previous_preferences.show_charts;
    preferences.show_chart_threshold_lines = previous_preferences.show_chart_threshold_lines;
    preferences.editor_preferences = previous_preferences.editor_preferences.clone();
    preferences.auto_refresh_interval = previous_preferences.auto_refresh_interval;
    preferences.cert_monitor_refresh_interval = previous_preferences.cert_monitor_refresh_interval;
    if preferences != *previous_preferences || hosts_config.certificate_monitors != previous_hosts_config.certificate_monitors {
        return false;
    }

    if main_config.preferences.auto_refresh_interval != previous_preferences.auto_refresh_interval
        || main_config.preferences.cert_monitor_refresh_interval != previous_preferences.cert_monitor_refresh_interval
    {
        monitor_manager.start_timers(
            hosts_config,
            main_config.preferences.auto_refresh_interval,
            main_config.preferences.cert_monitor_refresh_interval,
            host_manager.frontend_update_senders(),
        );
    }

    let mut runtime_hosts = hosts_config.clone();
    Configuration::resolve_secrets_in_hosts(&mut runtime_hosts, secret_store);
    host_manager.apply_monitors_acknowledged(&runtime_hosts)
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
    core.reconfigure(main_config, hosts_config, secret_store, true);

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
    let core = Rc::new(RefCell::new(
        initialize_core(
            main_config,
            hosts_config,
            Arc::new(ModuleFactory::new()),
            secret_store.clone(),
        )
        .map_err(String::from)?,
    ));

    let module_metadatas = core.borrow().module_factory.get_module_metadatas();
    let command_backend: Box<dyn backend::CommandBackend> =
        Box::new(backend::LocalCommandBackend::new(core.clone()));
    let config_backend: Box<dyn backend::ConfigBackend> =
        Box::new(backend::LocalConfigBackend::new(config_dir.clone(), secret_store.clone()));

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
    core.borrow().host_manager.borrow_mut().add_observer(frontend.new_update_sender());
    if !test {
        Ok(frontend.start(
            command_backend,
            config_backend,
            core,
            secret_store,
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
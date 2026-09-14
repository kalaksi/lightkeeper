/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

extern crate qmetaobject;
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::mpsc, sync::Arc, thread};

use qmetaobject::*;

use crate::{
    backend::{
        probe_admin_host, AdminHostProbe, CoreConnectionState, RemoteCommandBackend,
        RemoteConfigBackend, RemoteCoreClient,
    },
    configuration::{Configuration, CoreConnectionProfile},
    connection_manager::ConnectionManager,
    file_handler,
    frontend::{HostDisplayData, UIUpdate},
    host_manager,
    module::monitoring::{DataPoint, MonitoringData},
    metrics,
};

use super::{
    CommandHandlerModel,
    ConfigManagerModel,
    HostDataManagerModel,
    MetricsManagerModel
};


// This should probably be renamed to something like RequestHandlerModel.
#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct LkBackend {
    base: qt_base_class!(trait QObject),

    pub command: qt_property!(RefCell<CommandHandlerModel>; CONST),
    pub hosts: qt_property!(RefCell<HostDataManagerModel>; CONST),
    pub config: qt_property!(RefCell<ConfigManagerModel>; CONST),
    pub metrics: qt_property!(RefCell<MetricsManagerModel>; CONST),

    //
    // Slots
    //

    receiveUpdates: qt_method!(fn(&self)),
    reload: qt_method!(fn(&mut self)),
    acknowledgeMonitorEntry: qt_method!(fn(&mut self, host_id: QString, monitor_id: QString, entry_id: QString, acknowledged: bool) -> bool),
    stop: qt_method!(fn(&mut self)),

    probeCore: qt_method!(fn(&mut self) -> QString),
    probeCoreHost: qt_method!(fn(&mut self) -> QString),
    getCoreHostProbe: qt_method!(fn(&self) -> QVariantMap),
    connectCore: qt_method!(fn(&mut self) -> QString),
    disconnectCore: qt_method!(fn(&mut self)),
    getCoreConnectionState: qt_method!(fn(&self) -> QString),
    getCoreConnectionError: qt_method!(fn(&self) -> QString),
    isUsingRemoteCore: qt_method!(fn(&self) -> bool),

    //
    // Signals
    //

    reloaded: qt_signal!(error: QString, reset_hosts: QStringList),
    // Somewhere, a panic has occurred and app needs to be reloaded.
    crashed: qt_signal!(),
    coreConnectionChanged: qt_signal!(),

    //
    // Private properties
    //

    update_sender_prototype: Option<mpsc::Sender<UIUpdate>>,
    update_receiver: Option<mpsc::Receiver<UIUpdate>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,

    connection_manager: ConnectionManager,
    host_manager: Rc<RefCell<host_manager::HostManager>>,
    skip_connection_processing: bool,
    remote_client: Option<Arc<RemoteCoreClient>>,
    using_remote: bool,
    last_admin_host_probe: Option<(CoreConnectionProfile, AdminHostProbe)>,
}

#[allow(non_snake_case)]
impl LkBackend {
    pub fn new(
        update_sender_prototype: mpsc::Sender<UIUpdate>,
        update_receiver: mpsc::Receiver<UIUpdate>,
        host_manager: Rc<RefCell<host_manager::HostManager>>,
        connection_manager: ConnectionManager,
        host_data_model: HostDataManagerModel,
        command_model: CommandHandlerModel,
        metrics_model: MetricsManagerModel,
        config_model: ConfigManagerModel,
        skip_connection_processing: bool,
    ) -> LkBackend {

        LkBackend {
            hosts: RefCell::new(host_data_model),
            command: RefCell::new(command_model),
            config: RefCell::new(config_model),
            metrics: RefCell::new(metrics_model),
            update_sender_prototype: Some(update_sender_prototype),
            update_receiver: Some(update_receiver),
            update_receiver_thread: None,
            host_manager: host_manager,
            connection_manager: connection_manager,
            skip_connection_processing,
            ..Default::default()
        }
    }

    fn receiveUpdates(&mut self) {
        // Shouldn't (and can't) be run more than once.
        let update_receiver = if let Some(receiver) = self.update_receiver.take() {
            receiver
        } else {
            return;
        };

        let self_ptr = QPointer::from(&*self);
        let process_host_update = qmetaobject::queued_callback(move |new_data: HostDisplayData| {
            if let Some(self_pinned) = self_ptr.as_pinned() {

                // Special case for host initialization. Proper monitor processing is started after initialization step.
                if new_data.host_state.just_initialized {
                    ::log::debug!("Host {} initialized", new_data.host_state.host.name);
                    self_pinned.borrow().command.borrow_mut().refresh_host_monitors(new_data.host_state.host.name);
                    return;
                }

                self_pinned.borrow().hosts.borrow_mut().process_update(new_data);
            }
        });

        let self_ptr = QPointer::from(&*self);
        let process_chart_update = qmetaobject::queued_callback(move |response: metrics::lmserver::LMSResponse| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
                self_pinned.borrow().metrics.borrow_mut().process_update(response);
            }
        });

        let self_ptr = QPointer::from(&*self);
        let process_chart_insert = qmetaobject::queued_callback(move |(host_id, new_monitoring_data): (String, MonitoringData)| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
                for data_point in new_monitoring_data.values {
                    self_pinned.borrow().metrics.borrow_mut().insert_data_point(&host_id, &new_monitoring_data.monitor_id, data_point);
                }
            }
        });

        let self_ptr = QPointer::from(&*self);
        let process_alert_insert = qmetaobject::queued_callback(move |(
            host_id,
            monitor_id,
            previous,
            current,
            use_multivalue,
            root_label,
        ): (String, String, Option<DataPoint>, DataPoint, bool, String)| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
                self_pinned.borrow().metrics.borrow_mut().insert_alert_transitions(
                    &host_id,
                    &monitor_id,
                    previous.as_ref(),
                    &current,
                    use_multivalue,
                    &root_label,
                );
            }
        });

        let self_ptr = QPointer::from(&*self);
        let handle_crash = qmetaobject::queued_callback(move |_| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
                self_pinned.borrow().crashed();
            }
        });

        let self_ptr = QPointer::from(&*self);
        let handle_core_connection_change = qmetaobject::queued_callback(move |_| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
                self_pinned.borrow().coreConnectionChanged();
            }
        });

        let thread = std::thread::spawn(move || {
            loop {
                match update_receiver.recv() {
                    Ok(received_data) => {
                        match received_data {
                            UIUpdate::Host(display_data) => {
                                if let Some(new_monitoring_data) = &display_data.new_monitoring_data {
                                    let monitor_id = new_monitoring_data.1.monitor_id.clone();
                                    let previous = display_data.host_state.monitor_data
                                        .get(&monitor_id)
                                        .and_then(|monitor_data| {
                                            if monitor_data.values.len() >= 2 {
                                                monitor_data.values.get(monitor_data.values.len() - 2).cloned()
                                            }
                                            else {
                                                None
                                            }
                                        });
                                    if let Some(current) = new_monitoring_data.1.values.back().cloned() {
                                        process_alert_insert((
                                            display_data.host_state.host.name.clone(),
                                            monitor_id,
                                            previous,
                                            current,
                                            new_monitoring_data.1.display_options.use_multivalue,
                                            new_monitoring_data.1.display_options.display_text.clone(),
                                        ));
                                    }

                                    if new_monitoring_data.1.display_options.use_with_charts {
                                        process_chart_insert((
                                            display_data.host_state.host.name.clone(),
                                            new_monitoring_data.1.clone(),
                                        ));
                                    }
                                }
                                process_host_update(display_data);
                            }
                            UIUpdate::Chart(metrics) => process_chart_update(metrics),
                            UIUpdate::FatalError() => {
                                handle_crash(());
                                handle_core_connection_change(());
                            },
                            UIUpdate::Stop() => {
                                ::log::debug!("Gracefully exiting UI state receiver thread");
                                return;
                            }
                        };
                    },
                    Err(error) => {
                        ::log::error!("Stopped UI state receiver thread: {}", error);
                        return;
                    }
                }
            }
        });

        self.update_receiver_thread = Some(thread);
    }

    pub fn new_update_sender(&self) -> mpsc::Sender<UIUpdate> {
        self.update_sender_prototype.clone().unwrap()
    }

    fn local_core_socket_path() -> PathBuf {
        file_handler::get_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("core.sock")
    }

    fn detach_local_producers(&mut self) {
        self.command.borrow_mut().stop();
        self.host_manager.borrow_mut().stop();
        self.connection_manager.stop();
    }

    fn sync_models_from_config(&mut self) {
        let main_config = self.config.borrow().main_config_ref().clone();
        let hosts_config = self.config.borrow().hosts_config().clone();
        self.command.borrow_mut().set_configuration(main_config.clone());
        self.hosts.borrow_mut().set_hosts_config(hosts_config.clone());
        self.hosts.borrow_mut().set_configuration_preferences(main_config.preferences.clone());
        self.metrics.borrow_mut().set_hosts_config(hosts_config);
    }

    fn install_remote_backends(&mut self, client: Arc<RemoteCoreClient>) -> Result<(), String> {
        let config_backend = Box::new(RemoteConfigBackend::new(client.clone()));
        self.config.borrow_mut().set_config_backend(config_backend)?;
        self.command.borrow_mut().set_backend(Box::new(RemoteCommandBackend::new(client)));
        self.sync_models_from_config();
        Ok(())
    }

    fn probeCore(&mut self) -> QString {
        let profile = self.config.borrow().core_connection_profile();
        if !profile.is_configured() {
            return QString::from("SSH host is required");
        }

        let client = RemoteCoreClient::new(Self::local_core_socket_path());
        client.set_profile(profile);
        match client.probe() {
            Ok(()) => QString::from(""),
            Err(error) => QString::from(error),
        }
    }

    fn probeCoreHost(&mut self) -> QString {
        let profile = self.config.borrow().core_connection_profile();
        if !profile.is_configured() {
            self.last_admin_host_probe = None;
            return QString::from("SSH host is required");
        }

        let cancel = std::sync::atomic::AtomicBool::new(false);
        match probe_admin_host(&profile, &cancel) {
            Ok(probe) => {
                self.last_admin_host_probe = Some((profile, probe));
                QString::from("")
            }
            Err(error) => {
                self.last_admin_host_probe = None;
                QString::from(error)
            }
        }
    }

    fn getCoreHostProbe(&self) -> QVariantMap {
        let mut map = QVariantMap::default();
        let current_profile = self.config.borrow().core_connection_profile();
        let Some((profile, probe)) = &self.last_admin_host_probe else {
            return map;
        };
        if profile != &current_profile {
            return map;
        }

        map.insert("os".into(), QString::from(probe.platform.os.to_string()).into());
        map.insert(
            "osFlavor".into(),
            QString::from(probe.platform.os_flavor.to_string()).into(),
        );
        map.insert(
            "osVersion".into(),
            QString::from(probe.platform.os_version.to_string()).into(),
        );
        map.insert(
            "osVariantId".into(),
            QString::from(probe.platform.os_variant_id.clone()).into(),
        );
        map.insert(
            "architecture".into(),
            QString::from(probe.platform.architecture.to_string()).into(),
        );
        if let Some(socket_path) = &probe.socket_path {
            map.insert("socketPath".into(), QString::from(socket_path.clone()).into());
        }
        if let Some(binary_path) = &probe.binary_path {
            map.insert("binaryPath".into(), QString::from(binary_path.clone()).into());
        }
        map
    }

    fn connectCore(&mut self) -> QString {
        let profile = self.config.borrow().core_connection_profile();
        if !profile.is_configured() {
            return QString::from("SSH host is required");
        }

        // Validate reachability before tearing down the local backend.
        if !self.using_remote {
            let probe_client = RemoteCoreClient::new(Self::local_core_socket_path());
            probe_client.set_profile(profile.clone());
            if let Err(error) = probe_client.probe() {
                self.coreConnectionChanged();
                return QString::from(error);
            }
            self.detach_local_producers();
            self.using_remote = true;
        }
        else if let Some(client) = &self.remote_client {
            client.stop_connection();
        }

        let client = Arc::new(RemoteCoreClient::new(Self::local_core_socket_path()));
        client.set_profile(profile);
        client.set_frontend_update_sender(self.new_update_sender());

        if let Err(error) = client.connect() {
            self.remote_client = Some(client);
            self.hosts.borrow_mut().clear_hosts();
            self.coreConnectionChanged();
            self.reloaded(QString::from(error.clone()), QStringList::default());
            return QString::from(error);
        }

        if let Err(error) = self.install_remote_backends(client.clone()) {
            client.stop_connection();
            self.remote_client = Some(client);
            self.coreConnectionChanged();
            return QString::from(error);
        }

        self.hosts.borrow_mut().clear_hosts();
        self.remote_client = Some(client);
        self.coreConnectionChanged();
        self.reloaded(QString::from(""), QStringList::default());
        QString::from("")
    }

    fn disconnectCore(&mut self) {
        if let Some(client) = &self.remote_client {
            client.stop_connection();
        }
        self.hosts.borrow_mut().clear_hosts();
        self.coreConnectionChanged();
        self.reloaded(QString::from(""), QStringList::default());
    }

    fn getCoreConnectionState(&self) -> QString {
        let state = match &self.remote_client {
            Some(client) => client.connection_state(),
            None if self.using_remote => CoreConnectionState::Disconnected,
            None => CoreConnectionState::Disconnected,
        };
        QString::from(state.to_string())
    }

    fn getCoreConnectionError(&self) -> QString {
        match &self.remote_client {
            Some(client) => QString::from(client.last_error().unwrap_or_default()),
            None => QString::from(""),
        }
    }

    fn isUsingRemoteCore(&self) -> bool {
        self.using_remote
    }

    fn acknowledgeMonitorEntry(&mut self, host_id: QString, monitor_id: QString, entry_id: QString, acknowledged: bool) -> bool {
        let host = host_id.to_string();
        let monitor = monitor_id.to_string();
        let entry = entry_id.to_string();

        let Some(host_settings) = self.config.borrow_mut().set_monitor_entry_acknowledged(
            &host,
            &monitor,
            &entry,
            acknowledged,
        ) else {
            return false;
        };

        if !self.using_remote {
            self.host_manager.borrow().apply_monitor_acknowledged(&host, &monitor, host_settings);
        }
        true
    }

    fn reload(&mut self) {
        match self.config.borrow_mut().reload_configuration() {
            Ok((main_config, hosts_config)) => {
                if self.using_remote {
                    self.command.borrow_mut().set_configuration(main_config.clone());
                    self.hosts.borrow_mut().set_hosts_config(hosts_config.clone());
                    self.hosts.borrow_mut().set_configuration_preferences(main_config.preferences.clone());
                    self.metrics.borrow_mut().set_hosts_config(hosts_config);
                    self.reloaded(QString::from(""), QStringList::default());
                    return;
                }

                let mut runtime_hosts = hosts_config;
                Configuration::resolve_secrets_in_hosts(
                    &mut runtime_hosts,
                    Arc::new(crate::secrets_manager::KeyringSecretStore),
                );
                self.connection_manager.configure(&runtime_hosts);
                let reset_hosts = self.host_manager.borrow_mut().configure(&runtime_hosts);
                self.command.borrow_mut().configure(
                    &main_config,
                    &runtime_hosts,
                    self.connection_manager.new_request_sender(),
                    self.host_manager.borrow().new_state_update_sender(),
                    self.new_update_sender(),
                );

                // `self.metrics` doesn't have to be reconfigured.

                self.host_manager.borrow_mut().start_receiving_updates();
                if !self.skip_connection_processing {
                    self.connection_manager.start_processing_requests();
                }
                self.command.borrow_mut().start_processing_responses();

                let reset_hosts = reset_hosts.into_iter().map(QString::from).collect::<QStringList>();
                self.reloaded(QString::from(""), reset_hosts);
            },
            Err(error) => {
                let error = format!("Failed to reload configuration: {}", error);
                ::log::error!("{}", error);
                self.reloaded(QString::from(error), QStringList::default());
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(client) = &self.remote_client {
            client.stop_connection();
        }

        if let Some(thread) = self.update_receiver_thread.take() {
            if let Err(error) = self.new_update_sender().send(UIUpdate::Stop()) {
                ::log::error!("Failed to send stop signal to UI state receiver: {}", error);
            }

            if let Err(error) = thread.join() {
                ::log::error!("Error in thread: {:?}", error);
            }
        }

        self.command.borrow_mut().stop();
        if !self.using_remote {
            self.host_manager.borrow_mut().stop();
            self.connection_manager.stop();
        }
        self.metrics.borrow_mut().stop();
    }
}

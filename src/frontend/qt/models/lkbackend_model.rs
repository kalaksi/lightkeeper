/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

extern crate qmetaobject;
use std::{cell::Cell, cell::RefCell, path::PathBuf, rc::Rc, sync::mpsc, sync::Arc, thread};

use qmetaobject::*;

use crate::{
    apply_soft_config_update,
    backend::{
        accept_remote_core_host_key, install_remote_core, probe_remote_core_host, ConfigBackend, CoreConnectionState,
        HostKeyChallenge, RemoteCommandBackend, RemoteConfigBackend, RemoteCoreClient, RemoteCoreHostProbe,
        SshAuthError,
    },
    configuration::{self, Configuration},
    file_handler,
    frontend::{DisplayData, HostDisplayData, UIUpdate},
    module::command::CommandResult,
    module::monitoring::{DataPoint, MonitoringData},
    metrics,
    secrets_manager::SecretStore,
    CoreComponents,
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
    clearCoreHostProbe: qt_method!(fn(&mut self)),
    installCore: qt_method!(fn(&mut self) -> QString),
    connectCore: qt_method!(fn(&mut self) -> QString),
    cancelConnectCore: qt_method!(fn(&mut self)),
    dismissCoreConnection: qt_method!(fn(&mut self)),
    clearFailedCoreConnection: qt_method!(fn(&mut self)),
    disconnectCore: qt_method!(fn(&mut self)),
    getCoreConnectionState: qt_method!(fn(&self) -> QString),
    getCoreConnectionError: qt_method!(fn(&self) -> QString),
    isCoreVersionMismatch: qt_method!(fn(&self) -> bool),
    isUsingRemoteCore: qt_method!(fn(&self) -> bool),
    isCoreAutoConnectSkipped: qt_method!(fn(&self) -> bool),
    getCoreHostKeyChallenge: qt_method!(fn(&self) -> QVariantMap),
    clearCoreHostKeyChallenge: qt_method!(fn(&mut self)),
    verifyCoreHostKey: qt_method!(fn(&mut self, key_id: QString) -> QString),

    //
    // Signals
    //

    reloaded: qt_signal!(error: QString, reset_hosts: QStringList),
    // Somewhere, a panic has occurred and app needs to be reloaded.
    crashed: qt_signal!(),
    coreConnectionChanged: qt_signal!(),
    // App should quit so it can be restarted in local mode.
    restartRequested: qt_signal!(),
    /// Seconds left on core-owned refresh timers. 0 means the trigger is due (or inactive).
    /// Whether a timer is configured off comes from the refresh-interval preference, not this signal.
    timersChanged: qt_signal!(hosts_secs_left: u64, certs_secs_left: u64),

    //
    // Private properties
    //

    update_sender_prototype: Option<mpsc::Sender<UIUpdate>>,
    update_receiver: Option<mpsc::Receiver<UIUpdate>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,

    /// Local-mode managers. Shared with LocalCommandBackend; unused while using remote core.
    core: Option<Rc<RefCell<CoreComponents>>>,
    secret_store: Option<Arc<dyn SecretStore>>,
    /// Last config applied to local managers (for soft reload), like CoreRuntime::applied_config.
    applied_config: (Configuration, configuration::Hosts),
    skip_connection_processing: bool,
    skip_core_auto_connect: bool,
    /// Shared with QmlFrontend, which restarts the app in local mode when set.
    restart_local: Rc<Cell<bool>>,
    remote_client: Option<Arc<RemoteCoreClient>>,
    using_remote: bool,
    last_remote_core_host_probe: Option<RemoteCoreHostProbe>,
    last_host_key_challenge: Option<HostKeyChallenge>,
    /// Set by the latest probe or connect attempt.
    core_version_mismatch: bool,
}

#[allow(non_snake_case)]
impl LkBackend {
    pub fn new(
        update_sender_prototype: mpsc::Sender<UIUpdate>,
        update_receiver: mpsc::Receiver<UIUpdate>,
        core: Rc<RefCell<CoreComponents>>,
        secret_store: Arc<dyn SecretStore>,
        host_data_model: HostDataManagerModel,
        command_model: CommandHandlerModel,
        metrics_model: MetricsManagerModel,
        config_model: ConfigManagerModel,
        skip_connection_processing: bool,
        skip_core_auto_connect: bool,
        restart_local: Rc<Cell<bool>>,
    ) -> LkBackend {
        let applied_config = (config_model.main_config_ref().clone(), config_model.hosts_config().clone());
        LkBackend {
            hosts: RefCell::new(host_data_model),
            command: RefCell::new(command_model),
            config: RefCell::new(config_model),
            metrics: RefCell::new(metrics_model),
            update_sender_prototype: Some(update_sender_prototype),
            update_receiver: Some(update_receiver),
            update_receiver_thread: None,
            core: Some(core),
            secret_store: Some(secret_store),
            applied_config,
            skip_connection_processing,
            skip_core_auto_connect,
            restart_local,
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
        let process_hosts_replace = qmetaobject::queued_callback(move |display_data: DisplayData| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
                self_pinned.borrow().hosts.borrow_mut().replace_hosts(display_data);
            }
        });

        let self_ptr = QPointer::from(&*self);
        let process_command_result =
            qmetaobject::queued_callback(move |(invocation_id, command_result): (u64, CommandResult)| {
                if let Some(self_pinned) = self_ptr.as_pinned() {
                    self_pinned.borrow().hosts.borrow_mut().process_command_result(invocation_id, command_result);
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

        let self_ptr = QPointer::from(&*self);
        let process_timers = qmetaobject::queued_callback(move |(hosts_secs_left, certs_secs_left): (u64, u64)| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
                self_pinned.borrow().timersChanged(hosts_secs_left, certs_secs_left);
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
                            UIUpdate::ReplaceHosts(display_data) => process_hosts_replace(display_data),
                            UIUpdate::Timers { hosts_secs_left, certs_secs_left } => {
                                process_timers((hosts_secs_left, certs_secs_left))
                            }
                            UIUpdate::Chart(metrics) => process_chart_update(metrics),
                            UIUpdate::CommandResult(invocation_id, command_result) => {
                                process_command_result((invocation_id, command_result))
                            }
                            UIUpdate::CoreConnectionChanged() => handle_core_connection_change(()),
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
        if let Some(core) = &self.core {
            let mut core = core.borrow_mut();
            core.host_manager.borrow_mut().stop();
            core.connection_manager.stop();
        }
    }

    fn sync_models_from_config(&mut self) {
        let main_config = self.config.borrow().main_config_ref().clone();
        let hosts_config = self.config.borrow().hosts_config().clone();
        self.command.borrow_mut().set_configuration(main_config.clone());
        self.hosts.borrow_mut().set_hosts_config(hosts_config.clone());
        self.hosts.borrow_mut().set_configuration_preferences(main_config.preferences.clone());
        self.metrics.borrow_mut().set_hosts_config(hosts_config);
        self.metrics.borrow_mut().set_display_options(main_config.display_options);
    }

    fn install_remote_backends(
        &mut self,
        client: Arc<RemoteCoreClient>,
        config_backend: RemoteConfigBackend,
        config: (Configuration, configuration::Hosts, configuration::Groups),
    ) {
        self.config.borrow_mut().set_config_backend(Box::new(config_backend), config);
        let mut command_backend = RemoteCommandBackend::new(client.clone());
        command_backend.set_preferences(self.config.borrow().main_config_ref().preferences.clone());
        self.command.borrow_mut().set_backend(Box::new(command_backend));
        self.metrics.borrow_mut().stop();
        self.metrics.borrow_mut().set_remote_client(Some(client));
        self.sync_models_from_config();
    }

    fn probeCore(&mut self) -> QString {
        let profile = self.config.borrow().core_connection_profile();
        if !profile.is_configured() {
            return QString::from("SSH host is required");
        }

        let client = RemoteCoreClient::new(Self::local_core_socket_path());
        client.set_profile(profile);
        let result = client.probe();
        self.core_version_mismatch = client.status().version_mismatch;
        match result {
            Ok(()) => {
                self.last_host_key_challenge = None;
                QString::from("")
            }
            Err(error) => {
                self.last_host_key_challenge = client.host_key_challenge();
                QString::from(error)
            }
        }
    }

    fn probeCoreHost(&mut self) -> QString {
        let profile = self.config.borrow().core_connection_profile();
        if !profile.is_configured() {
            self.last_remote_core_host_probe = None;
            return QString::from("SSH host is required");
        }

        let cancel = std::sync::atomic::AtomicBool::new(false);
        match probe_remote_core_host(&profile, &cancel) {
            Ok(probe) => {
                self.last_remote_core_host_probe = Some(probe);
                self.last_host_key_challenge = None;
                QString::from("")
            }
            Err(SshAuthError::HostKeyUnverified(challenge)) => {
                self.last_remote_core_host_probe = None;
                let message = challenge.message.clone();
                self.last_host_key_challenge = Some(challenge);
                QString::from(message)
            }
            Err(error) => {
                self.last_remote_core_host_probe = None;
                self.last_host_key_challenge = None;
                QString::from(error.to_ui_message())
            }
        }
    }

    fn getCoreHostProbe(&self) -> QVariantMap {
        let mut map = QVariantMap::default();
        let Some(probe) = &self.last_remote_core_host_probe else {
            return map;
        };

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
        map.insert("lingerEnabled".into(), probe.linger_enabled.into());
        map.insert(
            "installBinaryPath".into(),
            QString::from(probe.install_plan.binary_path.clone()).into(),
        );
        map.insert(
            "installUnitPath".into(),
            QString::from(probe.install_plan.unit_path.clone()).into(),
        );
        map.insert(
            "installSocketPath".into(),
            QString::from(probe.install_plan.socket_path.clone()).into(),
        );
        map
    }

    fn clearCoreHostProbe(&mut self) {
        self.last_remote_core_host_probe = None;
    }

    fn installCore(&mut self) -> QString {
        let Some(probe) = &self.last_remote_core_host_probe else {
            return QString::from("Check the connection before installing");
        };
        let profile = self.config.borrow().core_connection_profile();
        let cancel = std::sync::atomic::AtomicBool::new(false);
        match install_remote_core(&profile, probe, &cancel) {
            Ok(()) => QString::from(""),
            Err(error) => QString::from(error.to_ui_message()),
        }
    }

    fn getCoreHostKeyChallenge(&self) -> QVariantMap {
        let mut map = QVariantMap::default();
        let Some(challenge) = &self.last_host_key_challenge else {
            return map;
        };
        map.insert("message".into(), QString::from(challenge.message.clone()).into());
        map.insert("keyId".into(), QString::from(challenge.key_id.clone()).into());
        map
    }

    fn clearCoreHostKeyChallenge(&mut self) {
        self.last_host_key_challenge = None;
    }

    fn verifyCoreHostKey(&mut self, key_id: QString) -> QString {
        let profile = self.config.borrow().core_connection_profile();
        if !profile.is_configured() {
            return QString::from("SSH host is required");
        }
        let key_id = key_id.to_string();
        let cancel = std::sync::atomic::AtomicBool::new(false);
        match accept_remote_core_host_key(&profile, &key_id, &cancel) {
            Ok(()) => {
                self.last_host_key_challenge = None;
                QString::from("")
            }
            Err(error) => QString::from(error.to_ui_message()),
        }
    }

    /// Starts remote core connection on a background thread. Progress and completion are signaled via
    /// `coreConnectionChanged`. A failed attempt leaves `failed` on the model until dismissed or cleared.
    fn connectCore(&mut self) -> QString {
        self.last_remote_core_host_probe = None;
        self.last_host_key_challenge = None;

        let profile = self.config.borrow().core_connection_profile();
        if !profile.is_configured() {
            return self.fail_connect_attempt("SSH host is required");
        }
        if let Some(client) = &self.remote_client {
            match client.connection_state() {
                CoreConnectionState::ConnectingSsh | CoreConnectionState::Handshaking => {
                    return QString::from("Already connecting");
                }
                _ => {}
            }
        }

        if self.using_remote {
            if let Some(client) = self.remote_client.take() {
                Self::stop_client_in_background(client);
            }
        }

        let (remote_update_sender, remote_update_receiver) = mpsc::channel();
        let client = Arc::new(RemoteCoreClient::new(Self::local_core_socket_path()));
        client.set_profile(profile);
        client.set_frontend_update_sender(remote_update_sender);
        client.set_connection_state(CoreConnectionState::ConnectingSsh);

        self.remote_client = Some(client.clone());
        self.emit_core_connection_changed();

        let self_ptr = QPointer::from(&*self);
        let finish = qmetaobject::queued_callback(move |(expected_client, result): (
            Arc<RemoteCoreClient>,
            Result<
                (mpsc::Receiver<UIUpdate>, (Configuration, configuration::Hosts, configuration::Groups)),
                String,
            >,
        )| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
                let reload_hosts = {
                    let mut backend = self_pinned.borrow_mut();
                    backend.finish_connect_core(expected_client, result)
                };
                self_pinned.borrow().coreConnectionChanged();
                if reload_hosts {
                    self_pinned.borrow().reloaded(QString::from(""), QStringList::default());
                }
            }
        });

        thread::spawn(move || {
            let result = match client.connect() {
                Ok(()) => {
                    let config_backend = RemoteConfigBackend::new(client.clone());
                    match config_backend.get_config() {
                        Ok(config) => Ok((remote_update_receiver, config)),
                        Err(error) => {
                            client.stop_connection();
                            Err(error.to_string())
                        }
                    }
                }
                Err(error) => Err(error),
            };
            finish((client, result));
        });

        QString::from("")
    }

    fn fail_connect_attempt(&mut self, message: &str) -> QString {
        let client = Arc::new(RemoteCoreClient::new(Self::local_core_socket_path()));
        client.set_profile(self.config.borrow().core_connection_profile());
        client.set_connection_failed(message);
        self.remote_client = Some(client);
        self.emit_core_connection_changed();
        QString::from(message)
    }

    /// Returns whether the host table should be reloaded after the attempt.
    fn finish_connect_core(
        &mut self,
        expected_client: Arc<RemoteCoreClient>,
        result: Result<
            (mpsc::Receiver<UIUpdate>, (Configuration, configuration::Hosts, configuration::Groups)),
            String,
        >,
    ) -> bool {
        let Some(client) = self.remote_client.clone() else {
            Self::stop_client_in_background(expected_client);
            return false;
        };
        if !Arc::ptr_eq(&client, &expected_client) {
            Self::stop_client_in_background(expected_client);
            return false;
        }
        self.core_version_mismatch = client.status().version_mismatch;

        match result {
            Ok((remote_update_receiver, remote_config)) => {
                if client.is_cancel_requested() {
                    Self::stop_cancelled_client_in_background(client, self.new_update_sender());
                    return false;
                }
                self.last_host_key_challenge = None;
                if !self.using_remote {
                    self.detach_local_producers();
                }

                let config_backend = RemoteConfigBackend::new(client.clone());
                self.install_remote_backends(client.clone(), config_backend, remote_config);
                self.hosts.borrow_mut().clear_hosts();
                self.using_remote = true;
                let frontend_update_sender = self.new_update_sender();
                thread::spawn(move || {
                    while let Ok(update) = remote_update_receiver.recv() {
                        if frontend_update_sender.send(update).is_err() {
                            return;
                        }
                    }
                });
                true
            }
            Err(_) => {
                self.last_host_key_challenge = client.host_key_challenge();
                if self.using_remote {
                    self.hosts.borrow_mut().clear_hosts();
                    true
                }
                else {
                    false
                }
            }
        }
    }

    fn cancelConnectCore(&mut self) {
        let Some(client) = self.remote_client.clone() else {
            return;
        };
        match client.connection_state() {
            CoreConnectionState::ConnectingSsh | CoreConnectionState::Handshaking => {}
            _ => return,
        }
        client.cancel_connect();
        // Immediate UI feedback; SSH teardown runs in the background (stop can block on wait_close).
        client.set_connection_failed("SSH core connection cancelled");
        self.emit_core_connection_changed();
        Self::stop_cancelled_client_in_background(client, self.new_update_sender());
    }

    /// Leave a failed/cancelled connect overlay, or restart locally if remote backends are active.
    fn dismissCoreConnection(&mut self) {
        if self.using_remote {
            self.disconnectCore();
            return;
        }
        if let Some(client) = self.remote_client.take() {
            client.cancel_connect();
            Self::stop_client_in_background(client);
        }
        self.last_host_key_challenge = None;
        self.emit_core_connection_changed();
    }

    /// Drop a sticky failed connect attempt so the main overlay hides, without clearing host-key /
    /// version-mismatch info the connection dialog may still need.
    fn clearFailedCoreConnection(&mut self) {
        if self.using_remote {
            return;
        }
        let Some(client) = self.remote_client.take() else {
            return;
        };
        if client.connection_state() != CoreConnectionState::Failed {
            self.remote_client = Some(client);
            return;
        }
        Self::stop_client_in_background(client);
        self.emit_core_connection_changed();
    }

    fn stop_client_in_background(client: Arc<RemoteCoreClient>) {
        thread::spawn(move || {
            client.stop_connection();
        });
    }

    /// stop_connection() sets Disconnected; restore Failed so the blocking overlay stays visible.
    fn stop_cancelled_client_in_background(client: Arc<RemoteCoreClient>, update_sender: mpsc::Sender<UIUpdate>) {
        thread::spawn(move || {
            client.stop_connection();
            client.set_connection_failed("SSH core connection cancelled");
            let _ = update_sender.send(UIUpdate::CoreConnectionChanged());
        });
    }

    fn emit_core_connection_changed(&self) {
        // Route through the UI update thread so the signal is always delivered asynchronously.
        let _ = self.new_update_sender().send(UIUpdate::CoreConnectionChanged());
    }

    /// Local backends were detached when switching to remote, so the app is restarted to use them again.
    fn disconnectCore(&mut self) {
        if let Some(client) = &self.remote_client {
            client.stop_connection();
        }
        self.restart_local.set(true);
        self.restartRequested();
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

    fn isCoreVersionMismatch(&self) -> bool {
        self.core_version_mismatch
    }

    fn isUsingRemoteCore(&self) -> bool {
        self.using_remote
    }

    fn isCoreAutoConnectSkipped(&self) -> bool {
        self.skip_core_auto_connect
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
            if let Some(core) = &self.core {
                core.borrow()
                    .host_manager
                    .borrow()
                    .apply_monitor_acknowledged(&host, &monitor, host_settings);
            }
        }
        true
    }

    fn reload(&mut self) {
        let reload_result = self.config.borrow_mut().reload_configuration();
        match reload_result {
            Ok((main_config, hosts_config, remote_reset_hosts)) => {
                let reset_hosts = if self.using_remote {
                    self.command.borrow_mut().set_configuration(main_config.clone());
                    self.hosts.borrow_mut().set_hosts_config(hosts_config.clone());
                    self.hosts.borrow_mut().set_configuration_preferences(main_config.preferences.clone());
                    self.metrics.borrow_mut().set_hosts_config(hosts_config);
                    self.metrics.borrow_mut().set_display_options(main_config.display_options);
                    remote_reset_hosts
                }
                else {
                    self.reload_local(main_config, hosts_config)
                };

                self.command.borrow_mut().initialize_reset_hosts(&reset_hosts);
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

    /// Soft-apply when possible (like remote ApplyConfig), otherwise CoreComponents::reconfigure.
    fn reload_local(&mut self, main_config: Configuration, hosts_config: configuration::Hosts) -> Vec<String> {
        let core = self.core.as_ref().expect("local core missing").clone();
        let secret_store = self.secret_store.as_ref().expect("local secret store missing").clone();
        let (previous_main, previous_hosts) = &self.applied_config;
        let soft_applied = {
            let mut core = core.borrow_mut();
            let host_manager = core.host_manager.clone();
            let soft_applied = apply_soft_config_update(
                &host_manager.borrow(),
                &mut core.monitor_manager,
                secret_store.clone(),
                previous_main,
                previous_hosts,
                &main_config,
                &hosts_config,
            );
            soft_applied
        };

        let reset_hosts = if soft_applied {
            Vec::new()
        }
        else {
            core.borrow_mut().reconfigure(
                &main_config,
                &hosts_config,
                secret_store,
                !self.skip_connection_processing,
            )
        };

        self.command.borrow_mut().set_configuration(main_config.clone());
        if soft_applied {
            self.hosts.borrow_mut().set_hosts_config(hosts_config.clone());
            self.hosts.borrow_mut().set_configuration_preferences(main_config.preferences.clone());
        }
        else {
            let display_data = core.borrow().host_manager.borrow().get_display_data();
            self.hosts.borrow_mut().sync_after_reload(
                display_data,
                hosts_config.clone(),
                main_config.preferences.clone(),
            );
        }
        self.metrics.borrow_mut().set_hosts_config(hosts_config.clone());
        self.metrics.borrow_mut().set_display_options(main_config.display_options.clone());
        self.applied_config = (main_config, hosts_config);
        reset_hosts
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
            if let Some(core) = &self.core {
                let mut core = core.borrow_mut();
                core.host_manager.borrow_mut().stop();
                core.connection_manager.stop();
            }
        }
        self.metrics.borrow_mut().stop();
    }
}

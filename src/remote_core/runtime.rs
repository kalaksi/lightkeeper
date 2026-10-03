/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::configuration;
use crate::error::LkError;
use crate::file_handler;
use crate::frontend;
use crate::metrics::MetricsManager;
use crate::secrets_manager::{KeyringSecretStore, SecretStore};
use crate::Configuration;
use crate::CoreComponents;
use crate::ModuleFactory;

pub struct CoreRuntime {
    pub core: CoreComponents,
    pub config_dir: String,
    pub secret_store: Arc<dyn SecretStore>,
    metrics_manager: Option<Arc<Mutex<MetricsManager>>>,
    chart_session_sender: Arc<Mutex<Option<mpsc::Sender<frontend::UIUpdate>>>>,
}

impl CoreRuntime {
    pub fn new(main_config: &Configuration, hosts_config: &configuration::Hosts, config_dir: String) -> Result<Self, LkError> {
        Self::new_with(
            main_config,
            hosts_config,
            Arc::new(ModuleFactory::new()),
            config_dir,
            Arc::new(KeyringSecretStore),
        )
    }

    pub fn new_with(
        main_config: &Configuration,
        hosts_config: &configuration::Hosts,
        module_factory: Arc<ModuleFactory>,
        config_dir: String,
        secret_store: Arc<dyn SecretStore>,
    ) -> Result<Self, LkError> {
        let mut core = crate::initialize_core(main_config, hosts_config, module_factory, secret_store.clone())?;

        if main_config.preferences.refresh_hosts_on_start {
            let host_ids = core.monitor_manager.refresh_platform_info_all();
            log::info!("Initialized {} host(s)", host_ids.len());
        }

        let chart_session_sender = Arc::new(Mutex::new(None));
        let metrics_manager = if main_config.preferences.show_charts {
            Some(Self::start_metrics(chart_session_sender.clone())?)
        }
        else {
            log::debug!("Charts are disabled.");
            None
        };

        let mut runtime = CoreRuntime {
            core,
            config_dir,
            secret_store,
            metrics_manager,
            chart_session_sender,
        };
        runtime.attach_metrics_ingest();

        Ok(runtime)
    }

    pub fn default_socket_path() -> Result<PathBuf, LkError> {
        Ok(file_handler::get_data_dir()?.join("core.sock"))
    }

    pub fn new_update_receiver(&mut self) -> mpsc::Receiver<frontend::UIUpdate> {
        let (sender, receiver) = mpsc::channel();
        self.core.host_manager.borrow_mut().add_observer(sender.clone());
        *self.chart_session_sender.lock().unwrap() = Some(sender);
        receiver
    }

    pub fn clear_session_update_sender(&self) {
        *self.chart_session_sender.lock().unwrap() = None;
    }

    pub fn get_metrics(&self, host_id: &str, monitor_id: &str, start_time: i64, end_time: i64) -> Result<u64, LkError> {
        let Some(metrics_manager) = &self.metrics_manager else {
            return Err(LkError::other("Charts are disabled on the remote core"));
        };
        metrics_manager.lock().unwrap().get_metrics(host_id, monitor_id, start_time, end_time)
    }

    pub fn get_alerts(
        &self,
        host_id: &str,
        monitor_id: &str,
        search_text: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<u64, LkError> {
        let Some(metrics_manager) = &self.metrics_manager else {
            return Err(LkError::other("Charts are disabled on the remote core"));
        };
        metrics_manager
            .lock()
            .unwrap()
            .get_alerts(host_id, monitor_id, search_text, start_time, end_time)
    }

    /// Restarts metrics ingest after core components are replaced (e.g. config update).
    pub fn attach_metrics_ingest(&mut self) {
        let Some(metrics_manager) = self.metrics_manager.clone() else {
            return;
        };

        let (sender, receiver) = mpsc::channel();
        self.core.host_manager.borrow_mut().add_observer(sender);

        thread::spawn(move || {
            while let Ok(update) = receiver.recv() {
                let frontend::UIUpdate::Host(display_data) = update else {
                    continue;
                };

                let Some(new_monitoring_data) = display_data.new_monitoring_data else {
                    continue;
                };

                let monitor_id = new_monitoring_data.1.monitor_id.clone();
                let previous = display_data
                    .host_state
                    .monitor_data
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
                    let mut metrics_manager = metrics_manager.lock().unwrap();
                    if let Err(error) = metrics_manager.insert_alert_transitions(
                        &display_data.host_state.host.name,
                        &monitor_id,
                        previous.as_ref(),
                        &current,
                        new_monitoring_data.1.display_options.use_multivalue,
                        &new_monitoring_data.1.display_options.display_text,
                    ) {
                        log::error!("Error inserting alert history: {}", error);
                    }
                }

                if new_monitoring_data.1.display_options.use_with_charts {
                    let mut metrics_manager = metrics_manager.lock().unwrap();
                    for data_point in &new_monitoring_data.1.values {
                        if let Err(error) = metrics_manager.insert_data_point(
                            &display_data.host_state.host.name,
                            &new_monitoring_data.1.monitor_id,
                            data_point,
                        ) {
                            log::error!("Error inserting chart data point: {}", error);
                        }
                    }
                }
            }
        });
    }

    /// Align metrics service with preferences after configuration reload.
    pub fn refresh_metrics(&mut self, main_config: &Configuration) -> Result<(), LkError> {
        match (self.metrics_manager.is_some(), main_config.preferences.show_charts) {
            (false, true) => {
                self.metrics_manager = Some(Self::start_metrics(self.chart_session_sender.clone())?);
                self.attach_metrics_ingest();
            }
            (true, false) => {
                if let Some(metrics_manager) = self.metrics_manager.take() {
                    if let Err(error) = metrics_manager.lock().unwrap().stop() {
                        log::error!("Error stopping metrics server: {}", error);
                    }
                }
            }
            (true, true) => {
                self.attach_metrics_ingest();
            }
            (false, false) => {}
        }
        Ok(())
    }

    fn start_metrics(
        chart_session_sender: Arc<Mutex<Option<mpsc::Sender<frontend::UIUpdate>>>>,
    ) -> Result<Arc<Mutex<MetricsManager>>, LkError> {
        let (metrics_update_sender, metrics_update_receiver) = mpsc::channel();
        thread::spawn(move || {
            while let Ok(update) = metrics_update_receiver.recv() {
                if let Some(sender) = chart_session_sender.lock().unwrap().as_ref() {
                    let _ = sender.send(update);
                }
            }
        });

        let mut metrics_manager = MetricsManager::new(metrics_update_sender);
        if let Err(error) = metrics_manager.start_service() {
            log::error!("Failed to start metrics server. Charts will not be available: {}", error);
        }

        Ok(Arc::new(Mutex::new(metrics_manager)))
    }

    pub fn stop(&mut self) {
        self.clear_session_update_sender();
        self.core.command_handler.stop();
        self.core.monitor_manager.stop();
        self.core.host_manager.borrow_mut().stop();
        self.core.connection_manager.stop();
    }
}

impl Drop for CoreRuntime {
    fn drop(&mut self) {
        if let Some(metrics_manager) = self.metrics_manager.take() {
            if let Err(error) = metrics_manager.lock().unwrap().stop() {
                log::error!("Error stopping metrics server: {}", error);
            }
        }
        self.stop();
    }
}

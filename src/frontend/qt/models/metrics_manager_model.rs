/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


extern crate qmetaobject;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::SystemTime;

use qmetaobject::*;

use crate::{
    backend::RemoteCoreClient,
    configuration,
    enums::Criticality,
    metrics,
    module::monitoring::DataPoint,
};


#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct MetricsManagerModel {
    base: qt_base_class!(trait QObject),

    //
    // Slots
    //
    startService: qt_method!(fn(&self) -> ()),
    refreshCharts: qt_method!(fn(&self, host_id: QString, monitor_id: QString, start_time_sec: i64, end_time_sec: i64) -> u64),
    refreshAlertHistory: qt_method!(fn(&self, search_text: QString) -> u64),
    getCategories: qt_method!(fn(&self, host_id: QString) -> QStringList),
    getCategoryMonitorIds: qt_method!(fn(&self, host_id: QString, category_id: QString) -> QStringList),


    //
    // Signals
    //
    dataReceived: qt_signal!(invocation_id: u64, chart_data: QString),
    alertHistoryReceived: qt_signal!(invocation_id: u64, alert_data: QString),

    //
    // Private properties
    //
    metrics_manager: Option<metrics::MetricsManager>,
    remote_client: Option<Arc<RemoteCoreClient>>,
    hosts_config: configuration::Hosts,
    display_options: configuration::DisplayOptions,
    pending_alert_queries: HashSet<u64>,
}

#[allow(non_snake_case)]
impl MetricsManagerModel {
    pub fn new(
        metrics_manager: Option<metrics::MetricsManager>,
        hosts_config: configuration::Hosts,
        display_options: configuration::DisplayOptions) -> MetricsManagerModel {

        MetricsManagerModel {
            metrics_manager: metrics_manager,
            hosts_config: hosts_config,
            display_options: display_options,
            ..Default::default()
        }
    }

    pub fn stop(&mut self) {
        if let Some(metrics_manager) = self.metrics_manager.as_mut() {
            // TODO: notify UI?
            if let Err(error) = metrics_manager.stop() {
                ::log::error!("Error stopping metrics server: {}", error);
            }
        }
    }

    pub fn set_hosts_config(&mut self, hosts_config: configuration::Hosts) {
        self.hosts_config = hosts_config;
    }

    pub fn set_display_options(&mut self, display_options: configuration::DisplayOptions) {
        self.display_options = display_options;
    }

    pub fn set_remote_client(&mut self, remote_client: Option<Arc<RemoteCoreClient>>) {
        self.remote_client = remote_client;
    }

    pub fn insert_data_point(&mut self, host_id: &str, monitor_id: &str, data_point: DataPoint) {
        if self.remote_client.is_some() {
            return;
        }

        if let Some(metrics_manager) = self.metrics_manager.as_mut() {
            if let Err(error) = metrics_manager.insert_data_point(host_id, monitor_id, &data_point) {
                ::log::error!("Error inserting data point: {}", error);
            }
        }
    }

    pub fn insert_alert_transitions(
        &mut self,
        host_id: &str,
        monitor_id: &str,
        previous: Option<&DataPoint>,
        current: &DataPoint,
        use_multivalue: bool,
        root_label: &str,
    ) {
        if self.remote_client.is_some() {
            return;
        }

        let Some(metrics_manager) = self.metrics_manager.as_mut() else {
            return;
        };

        if let Err(error) = metrics_manager.insert_alert_transitions(
            host_id,
            monitor_id,
            previous,
            current,
            use_multivalue,
            root_label,
        ) {
            ::log::error!("Error inserting alert history: {}", error);
        }
    }

    pub fn process_update(&mut self, response: metrics::lmserver::LMSResponse) {
        if self.pending_alert_queries.remove(&response.request_id) {
            let alert_data = Self::alert_history_json(&response.alerts);
            self.alertHistoryReceived(response.request_id.into(), alert_data.into());
            return;
        }

        let chart_data = serde_json::to_string(&response.metrics).unwrap();
        self.dataReceived(response.request_id.into(), chart_data.into());
    }

    fn alert_history_json(events: &[metrics::lmserver::AlertEvent]) -> String {
        #[derive(serde::Serialize)]
        struct AlertHistoryItem<'a> {
            time: i64,
            host_id: &'a str,
            monitor_id: &'a str,
            entry_id: &'a str,
            label: &'a str,
            value: &'a str,
            from_level: String,
            to_level: String,
        }

        let mut items = events.iter().map(|event| {
            AlertHistoryItem {
                time: event.time,
                host_id: &event.host_id,
                monitor_id: &event.monitor_id,
                entry_id: &event.entry_id,
                label: &event.label,
                value: &event.value,
                from_level: Criticality::try_from(event.from_level)
                    .map(|level| level.to_string())
                    .unwrap_or_else(|_| event.from_level.to_string()),
                to_level: Criticality::try_from(event.to_level)
                    .map(|level| level.to_string())
                    .unwrap_or_else(|_| event.to_level.to_string()),
            }
        }).collect::<Vec<_>>();
        items.reverse();
        items.truncate(100);

        serde_json::to_string(&items).unwrap_or_else(|_| String::from("[]"))
    }

    fn startService(&mut self) {
        if self.remote_client.is_some() {
            return;
        }

        if let Some(metrics_manager) = self.metrics_manager.as_mut() {
            if let Err(error) = metrics_manager.start_service() {
                // TODO: show in UI
                ::log::error!("Error: {}", error);
                ::log::error!("Failed to start metrics server. Charts will not be available.");
            }
        }
    }

    fn refreshCharts(&mut self, host_id: QString, monitor_id: QString, start_time_sec: i64, end_time_sec: i64) -> u64 {
        if let Some(remote_client) = &self.remote_client {
            return match remote_client.get_metrics(
                &host_id.to_string(),
                &monitor_id.to_string(),
                start_time_sec,
                end_time_sec,
            ) {
                Ok(invocation_id) => invocation_id,
                Err(error) => {
                    ::log::error!("Error refreshing charts: {}", error);
                    0
                }
            };
        }

        if let Some(metrics_manager) = self.metrics_manager.as_mut() {
            let invocation_result = metrics_manager.get_metrics(
                &host_id.to_string(),
                &monitor_id.to_string(),
                start_time_sec,
                end_time_sec,
            );

            match invocation_result {
                Ok(invocation_id) => invocation_id,
                Err(error) => {
                    ::log::error!("Error refreshing charts: {}", error);
                    0
                }
            }
        }
        else {
            0
        }
    }

    fn refreshAlertHistory(&mut self, search_text: QString) -> u64 {
        if let Some(remote_client) = &self.remote_client {
            let end_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(duration) => duration.as_secs() as i64,
                Err(_) => {
                    ::log::error!("Time calculation error");
                    return 0;
                }
            };
            let start_time = end_time - 24 * 60 * 60;

            return match remote_client.get_alerts("", "", &search_text.to_string(), start_time, end_time) {
                Ok(invocation_id) => {
                    self.pending_alert_queries.insert(invocation_id);
                    invocation_id
                }
                Err(error) => {
                    ::log::error!("Error refreshing alert history: {}", error);
                    0
                }
            };
        }

        let Some(metrics_manager) = self.metrics_manager.as_mut() else {
            return 0;
        };

        let end_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs() as i64,
            Err(_) => {
                ::log::error!("Time calculation error");
                return 0;
            }
        };
        let start_time = end_time - 24 * 60 * 60;

        match metrics_manager.get_alerts("", "", &search_text.to_string(), start_time, end_time) {
            Ok(invocation_id) => {
                self.pending_alert_queries.insert(invocation_id);
                invocation_id
            }
            Err(error) => {
                ::log::error!("Error refreshing alert history: {}", error);
                0
            }
        }
    }

    fn getCategories(&self, host_id: QString) -> QStringList {
        let host_id = host_id.to_string();
        let host_config = self.hosts_config.hosts.get(&host_id).unwrap();

        let host_monitors = host_config.effective.monitors.iter()
            .map(|(monitor_id, _config)| monitor_id)
            .collect::<Vec<_>>();

        // Filters out categories that don't have any monitors on this host.
        let categories = self.display_options.chart_categories.iter()
            .filter(|category| category.monitors.iter().any(|monitor_id| host_monitors.contains(&monitor_id)))
            .map(|category| category.name.clone());

        QStringList::from_iter(categories)
    }

    fn getCategoryMonitorIds(&self, host_id: QString, category_id: QString) -> QStringList {
        let host_id = host_id.to_string();
        let category_id = category_id.to_string();
        let host_config = match self.hosts_config.hosts.get(&host_id) {
            Some(config) => config,
            None => {
                ::log::error!("Host '{}' not found", host_id);
                return QStringList::new();
            }
        };

        let host_monitors = host_config.effective.monitors.iter()
            .map(|(monitor_id, _config)| monitor_id)
            .collect::<Vec<_>>();

        let category_monitors = self.display_options.chart_categories.iter()
            .find(|category| category.name == category_id)
            .map(|category| category.monitors.clone())
            .unwrap_or_default();

        // Intersection between host monitors and category monitors.
        let valid_monitors = host_monitors.into_iter()
            .filter(|monitor_id| category_monitors.contains(monitor_id))
            .cloned()
            .collect::<Vec<_>>();

        QStringList::from_iter(valid_monitors)
    }
}

/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


extern crate qmetaobject;
use std::time::SystemTime;

use qmetaobject::*;

use crate::{
    configuration,
    metrics,
    module::monitoring::DataPoint
};


#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct MetricsManagerModel {
    base: qt_base_class!(trait QObject),

    //
    // Slots
    //
    startService: qt_method!(fn(&self) -> ()),
    refreshCharts: qt_method!(fn(&self, host_id: QString, monitor_id: QString) -> u64),
    getCategories: qt_method!(fn(&self, host_id: QString) -> QStringList),
    getCategoryMonitorIds: qt_method!(fn(&self, host_id: QString, category_id: QString) -> QStringList),


    //
    // Signals
    //
    dataReceived: qt_signal!(invocation_id: u64, chart_data: QString),

    //
    // Private properties
    //
    metrics_manager: Option<metrics::MetricsManager>,
    hosts_config: configuration::Hosts,
    display_options: configuration::DisplayOptions,
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

    pub fn insert_data_point(&mut self, host_id: &str, monitor_id: &str, data_point: DataPoint) {
        if let Some(metrics_manager) = self.metrics_manager.as_mut() {
            let current_unix_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64;

            let mut metrics = vec![metrics::Metric {
                label: data_point.label.clone(),
                value: data_point.value_float,
                time: current_unix_ms
            }];

            for child in data_point.multivalue.iter() {
                metrics.push(metrics::Metric {
                    label: child.label.clone(),
                    value: child.value_float,
                    time: current_unix_ms
                });
            }

            if let Err(error) = metrics_manager.insert_metrics(host_id, monitor_id, &metrics) {
                ::log::error!("Error inserting data point: {}", error);
            }
        }
    }

    pub fn process_update(&mut self, response: metrics::lmserver::LMSResponse) {
        let chart_data = serde_json::to_string(&response.metrics).unwrap();
        self.dataReceived(response.request_id.into(), chart_data.into());
    }

    fn startService(&mut self) {
        if let Some(metrics_manager) = self.metrics_manager.as_mut() {
            if let Err(error) = metrics_manager.start_service() {
                // TODO: show in UI
                ::log::error!("Error: {}", error);
                ::log::error!("Failed to start metrics server. Charts will not be available.");
            }
        }
    }

    fn refreshCharts(&mut self, host_id: QString, monitor_id: QString) -> u64 {
        if let Some(metrics_manager) = self.metrics_manager.as_mut() {
            let current_unix_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

            let invocation_result = metrics_manager.get_metrics(
                &host_id.to_string(),
                &monitor_id.to_string(),
                // 4 days back.
                current_unix_time.as_secs() as i64 - (60 * 60 * 24 * 4),
                current_unix_time.as_secs() as i64,
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

    fn getCategories(&self, host_id: QString) -> QStringList {
        let host_id = host_id.to_string();
        let host_config = self.hosts_config.hosts.get(&host_id).unwrap();

        let host_monitors = host_config.effective.monitors.iter()
            .filter(|(_monitor_id, config)| config.enabled.unwrap_or(true))
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
            .filter(|(_monitor_id, config)| config.enabled.unwrap_or(true))
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

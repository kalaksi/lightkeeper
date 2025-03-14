extern crate qmetaobject;
use qmetaobject::*;

use crate::configuration;
use crate::enums::Criticality;
use crate::frontend;
use crate::module::monitoring::MonitoringData;


#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct HostDataManagerModel {
    base: qt_base_class!(trait QObject),
    monitorCriticalCount: qt_property!(u64; NOTIFY criticalityCountsChanged),
    monitorErrorCount: qt_property!(u64; NOTIFY criticalityCountsChanged),
    monitorWarningCount: qt_property!(u64; NOTIFY criticalityCountsChanged),
    monitorNormalCount: qt_property!(u64; NOTIFY criticalityCountsChanged),
    monitorNoDataCount: qt_property!(u64; NOTIFY criticalityCountsChanged),

    //
    // Signals
    //

    criticalityCountsChanged: qt_signal!(),

    updateReceived: qt_signal!(host_id: QString),
    monitorStateChanged: qt_signal!(host_id: QString, monitor_id: QString, new_criticality: QString),
    commandResultReceived: qt_signal!(command_result: QString, invocation_id: u64),
    monitoringDataReceived: qt_signal!(host_id: QString, category: QString, monitoring_data_qv: QVariant, invocation_id: u64),
    errorReceived: qt_signal!(criticality: QString, error: QString),
    verificationRequested: qt_signal!(host_id: QString, connector_id: QString, message: QString, key_id: QString),

    //
    // Slots
    //

    getMonitoringData: qt_method!(fn(&self, host_id: QString, monitor_id: QString) -> QVariant),
    getMonitoringDataJson: qt_method!(fn(&self, host_id: QString, monitor_id: QString) -> QString),
    getDisplayData: qt_method!(fn(&self) -> QVariant),
    getCategories: qt_method!(fn(&self, host_id: QString, ignore_empty: bool) -> QStringList),
    getCategoryMonitorIds: qt_method!(fn(&self, host_id: QString, category: QString) -> QStringList),
    refresh_hosts_on_start: qt_method!(fn(&self) -> bool),
    isHostInitialized: qt_method!(fn(&self, host_id: QString) -> bool),
    removeHost: qt_method!(fn(&self, host_id: QString)),

    getPendingMonitorCount: qt_method!(fn(&self, host_id: QString) -> u64),
    getPendingMonitorCountForCategory: qt_method!(fn(&self, host_id: QString, category: QString) -> u64),
    getPendingCommandCount: qt_method!(fn(&self, host_id: QString) -> u64),
    getPendingCommandCountForCategory: qt_method!(fn(&self, host_id: QString, category: QString) -> u64),
    getPendingCommandProgress: qt_method!(fn(&self, invocation_id: u64) -> u8),

    getCertificateMonitorHostId: qt_method!(fn(&self) -> QString),
    // These methods are used to get the data in JSON and parsed in QML side.
    // JSON is required since there doesn't seem to be a way to return a self-defined QObject.
    getSummaryMonitorData: qt_method!(fn(&self, host_id: QString) -> QStringList),
    getHostDataJson: qt_method!(fn(&self, host_id: QString) -> QString),

    //
    // Private properties
    //

    // Basically contains the state of hosts and relevant data.
    // Retains data between reloads.
    display_data: frontend::DisplayData,
    display_options_category_order: Vec<String>,
    configuration_preferences: configuration::Preferences,
    configuration_cache_settings: configuration::CacheSettings,
}

#[allow(non_snake_case)]
impl HostDataManagerModel {
    pub fn new(display_data: frontend::DisplayData, config: configuration::Configuration) -> Self {
        let mut priorities = config.display_options.categories.iter()
                                                              .map(|(category, options)| (category.clone(), options.priority))
                                                              .collect::<Vec<_>>();

        priorities.sort_by(|left, right| left.1.cmp(&right.1));

        let mut result = HostDataManagerModel {
            display_data: display_data,
            // display_options: display_options,
            display_options_category_order: priorities.into_iter().map(|(category, _)| category).collect(),
            configuration_preferences: config.preferences,
            configuration_cache_settings: config.cache_settings,
            ..Default::default()
        };

        result.update_criticality_counts();
        result
    }


    pub fn process_update(&mut self, new_display_data: frontend::HostDisplayData) {
        // HostDataModel cannot be passed between threads so parsing happens here.
        let host_state = &new_display_data.host_state;
        let maybe_old_data = self.display_data.hosts.insert(host_state.host.name.clone(), new_display_data.clone());

        if let Some((invocation_id, command_result)) = new_display_data.new_command_result {
            let json = QString::from(serde_json::to_string(&command_result).unwrap());
            self.commandResultReceived(json, invocation_id);
        }

        if let Some((invocation_id, new_monitor_data)) = new_display_data.new_monitoring_data {
            self.monitoringDataReceived(QString::from(host_state.host.name.clone()),
                                        QString::from(new_monitor_data.display_options.category.clone()),
                                        new_monitor_data.to_qvariant(),
                                        invocation_id);

            // Find out any monitor state changes and signal accordingly.
            if let Some(old_data) = maybe_old_data {
                let new_criticality = new_monitor_data.values.back().unwrap().criticality;

                if let Some(old_monitor_data) = old_data.host_state.monitor_data.get(&new_monitor_data.monitor_id) {
                    let old_criticality = old_monitor_data.values.back().unwrap().criticality;

                    if new_criticality != old_criticality {
                        self.monitorStateChanged(
                            QString::from(host_state.host.name.clone()),
                            QString::from(new_monitor_data.monitor_id.clone()),
                            QString::from(new_criticality.to_string())
                        );

                        self.update_criticality_counts();
                    }
                }
            }
        }

        for error in new_display_data.new_errors {
            self.errorReceived(QString::from(error.criticality.to_string()), QString::from(error.message));
        }

        for request in new_display_data.verification_requests {
            self.verificationRequested(
                QString::from(host_state.host.name.clone()),
                QString::from(request.source_id),
                QString::from(request.message),
                QString::from(request.key_id),
            );
        }

        self.updateReceived(QString::from(host_state.host.name.clone()));
    }

    // Get monitoring data as a QVariant.
    fn getMonitoringData(&self, host_id: QString, monitor_id: QString) -> QVariant {
        let host_id = host_id.to_string();
        let monitor_id = monitor_id.to_string();

        let display_data = self.display_data.hosts.get(&host_id).unwrap();
        display_data.host_state.monitor_data.get(&monitor_id).unwrap().to_qvariant()
    }

    fn getMonitoringDataJson(&self, host_id: QString, monitor_id: QString) -> QString {
        let host_id = host_id.to_string();
        let monitor_id = monitor_id.to_string();

        if let Some(display_data) = self.display_data.hosts.get(&host_id) {
            let monitor_data = display_data.host_state.monitor_data.get(&monitor_id).unwrap();
            QString::from(serde_json::to_string(&monitor_data).unwrap())
        }
        else {
            QString::from("{}")
        }
    }

    fn getDisplayData(&self) -> QVariant {
        self.display_data.to_qvariant()
    }

    // Get list of monitors for category.
    fn getCategoryMonitorIds(&self, host_id: QString, category: QString) -> QStringList {
        let host_id = host_id.to_string();
        let category = category.to_string();

        let mut result = QStringList::default();

        if let Some(display_data) = self.display_data.hosts.get(&host_id) {
            for (monitor_id, monitor_data) in display_data.host_state.monitor_data.iter() {
                if monitor_data.display_options.category == category {
                    result.push(QString::from(monitor_id.clone()));
                }
            }
        }

        result
    }

    fn refresh_hosts_on_start(&self) -> bool {
        self.configuration_preferences.refresh_hosts_on_start
    }

    fn isHostInitialized(&self, host_id: QString) -> bool {
        if let Some(display_data) = self.display_data.hosts.get(&host_id.to_string()) {
            // When reading host status from cache, existing platform info is enough for making it initialized.
            if self.configuration_cache_settings.enable_cache && self.configuration_cache_settings.prefer_cache {
                display_data.host_state.host.platform.is_set()
            }
            else {
                display_data.host_state.is_initialized
            }
        }
        else {
            false
        }
    }

    fn removeHost(&mut self, host_id: QString) {
        self.display_data.hosts.remove(&host_id.to_string());
    }

    fn getPendingMonitorCount(&self, host_id: QString) -> u64 {
        let host_id = host_id.to_string();

        if let Some(host_display_data) = self.display_data.hosts.get(&host_id) {
            host_display_data.host_state.monitor_invocations.len() as u64
        }
        else {
            0
        }
    }

    fn getPendingMonitorCountForCategory(&self, host_id: QString, category: QString) -> u64 {
        let host_id = host_id.to_string();
        let category = category.to_string();

        let host_display_data = self.display_data.hosts.get(&host_id).unwrap();
        host_display_data.host_state.monitor_invocations.values()
            .filter(|invocation| invocation.category == category)
            .count() as u64
    }

    fn getPendingCommandCount(&self, host_id: QString) -> u64 {
        let host_id = host_id.to_string();

        if let Some(host_display_data) = self.display_data.hosts.get(&host_id) {
            host_display_data.host_state.command_invocations.len() as u64
        }
        else {
            0
        }
    }

    fn getPendingCommandCountForCategory(&self, host_id: QString, category: QString) -> u64 {
        let host_id = host_id.to_string();
        let category = category.to_string();

        let host_display_data = self.display_data.hosts.get(&host_id).unwrap();
        host_display_data.host_state.command_invocations.values()
            .filter(|invocation| invocation.category == category)
            .count() as u64
    }

    fn getPendingCommandProgress(&self, invocation_id: u64) -> u8 {
        for host_display_data in self.display_data.hosts.values() {
            if let Some(command_invocation) = host_display_data.host_state.command_invocations.get(&invocation_id) {
                return command_invocation.progress;
            }
        }

        return 100;
    }

    fn is_empty_category(&self, host_id: &str, category: &str) -> bool {
        let display_data = self.display_data.hosts.get(host_id).unwrap();
        display_data.host_state.monitor_data.values()
            .filter(|monitor_data| monitor_data.display_options.category == category)
            .all(|monitor_data| monitor_data.values.iter()
                .all(|data_point| data_point.criticality == Criticality::Ignore || data_point.is_empty())
            )
    }

    /// Returns a readily sorted list of unique categories for a host. Gathered from monitoring data.
    fn getCategories(&self, host_id: QString, ignore_empty: bool) -> QStringList {
        let host_id = host_id.to_string();
        let display_data = self.display_data.hosts.get(&host_id.to_string()).unwrap();

        // Get unique categories from monitoring datas, and sort them according to config and alphabetically.
        let mut categories = display_data.host_state.monitor_data.values()
            .filter(|monitor_data| !monitor_data.display_options.category.is_empty())
            .map(|monitor_data| monitor_data.display_options.category.clone())
            .collect::<Vec<String>>();

        categories.sort();
        categories.dedup();

        let mut result = QStringList::default();

        // First add categories in the order they are defined in the config.
        for prioritized_category in self.display_options_category_order.iter() {
            if categories.contains(prioritized_category) &&
               !ignore_empty ||
               !self.is_empty_category(&host_id, &prioritized_category) {

                result.push(QString::from(prioritized_category.clone()));
            }
        }

        for remaining_category in categories.iter() {
            if !self.display_options_category_order.contains(remaining_category) {
                if !ignore_empty || !self.is_empty_category(&host_id, &remaining_category) {
                    result.push(QString::from(remaining_category.clone()));
                }
            }
        }

        result
    }

    fn getCertificateMonitorHostId(&self) -> QString {
        QString::from(crate::monitor_manager::CERT_MONITOR_HOST_ID)
    }

    fn getSummaryMonitorData(&self, host_id: QString) -> QStringList {
        let host_id = host_id.to_string();
        let mut result = QStringList::default();

        if let Some(display_data) = self.display_data.hosts.get(&host_id) {
            let overridden_monitors = display_data.host_state.monitor_data.values()
                .filter(|data| !data.display_options.override_summary_monitor_id.is_empty())
                .map(|data| &data.display_options.override_summary_monitor_id)
                .collect::<Vec<&String>>();

            let summary_compatible = display_data.host_state.monitor_data.values()
                .filter(|data| !data.display_options.use_without_summary && !overridden_monitors.contains(&&data.monitor_id))
                .collect();

            let sorted_keys = self.get_monitor_data_keys_sorted(summary_compatible);

            for key in sorted_keys {
                let monitoring_data = display_data.host_state.monitor_data.get(&key).unwrap();

                result.push(serde_json::to_string(&monitoring_data).unwrap().into());
            }
        }

        result
    }

    fn getHostDataJson(&self, host_id: QString) -> QString {
        // Doesn't include monitor and command datas.
        if let Some(display_data) = self.display_data.hosts.get(&host_id.to_string()) {
            let mut stripped = display_data.host_state.clone();
            stripped.monitor_data.clear();
            stripped.command_results.clear();
            QString::from(serde_json::to_string(&stripped).unwrap())
        }
        else {
            QString::from("{}")
        }
    }

    // Returns list of MonitorData structs in JSON. Empty if host doesn't exist.
    fn get_monitor_data_keys_sorted(&self, monitoring_data: Vec<&MonitoringData>) -> Vec<String> {
        let mut keys_ordered = Vec::<String>::new();

        // First include data of categories in an order that's defined in configuration.
        for category in self.display_options_category_order.iter() {
            let category_monitors = monitoring_data.iter().filter(|data| &data.display_options.category == category)
                                                          .collect::<Vec<&&MonitoringData>>();
            keys_ordered.extend(Self::sort_by_value_type(category_monitors));
        }

        let rest_of_monitors = monitoring_data.iter().filter(|data| !keys_ordered.contains(&data.monitor_id))
                                                     .collect::<Vec<&&MonitoringData>>();
        keys_ordered.extend(Self::sort_by_value_type(rest_of_monitors));

        keys_ordered
    }

    // Sorts first by value type (multivalue vs. single value) and then alphabetically.
    fn sort_by_value_type(datas: Vec<&&MonitoringData>) -> Vec<String> {
        let mut single_value_keys = datas.iter().filter(|data| !data.display_options.use_multivalue)
                                                .map(|data| data.monitor_id.clone())
                                                .collect::<Vec<String>>();
        single_value_keys.sort();

        let mut multivalue_keys = datas.iter().filter(|data| data.display_options.use_multivalue)
                                              .map(|data| data.monitor_id.clone())
                                              .collect::<Vec<String>>();
        multivalue_keys.sort();

        [ single_value_keys, multivalue_keys ].concat()
    }

    fn update_criticality_counts(&mut self) {
        let (mut criticals, mut errors, mut warnings, mut normals, mut nodatas) = (0, 0, 0, 0, 0);

        for monitor_data in self.display_data.hosts.values().flat_map(|host_data| host_data.host_state.monitor_data.values()) {
            let criticality = monitor_data.values.back().map(|data_point| data_point.criticality).unwrap_or(Criticality::NoData);

            match criticality {
                Criticality::Critical => criticals += 1,
                Criticality::Error => errors += 1,
                Criticality::Warning => warnings += 1,
                Criticality::Normal => normals += 1,
                Criticality::NoData => nodatas += 1,
                _ => {}
            }
        }

        let mut state_changed = false;
        if criticals != self.monitorCriticalCount {
            self.monitorCriticalCount = criticals;
            state_changed = true;
        }

        if errors != self.monitorErrorCount {
            self.monitorErrorCount = errors;
            state_changed = true;
        }

        if warnings != self.monitorWarningCount {
            self.monitorWarningCount = warnings;
            state_changed = true;
        }

        if normals != self.monitorNormalCount {
            self.monitorNormalCount = normals;
            state_changed = true;
        }

        if nodatas != self.monitorNoDataCount {
            self.monitorNoDataCount = nodatas;
            state_changed = true;
        }

        if state_changed {
            self.criticalityCountsChanged();
        }
    }
}
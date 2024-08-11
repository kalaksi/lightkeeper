extern crate qmetaobject;
use qmetaobject::*;

use crate::configuration;
use crate::enums::Criticality;
use crate::frontend;
use crate::module::monitoring::MonitoringData;
use crate::utils::ErrorMessage;


#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct HostDataManagerModel {
    base: qt_base_class!(trait QObject),
    reset: qt_method!(fn(&mut self)),

    //
    // Signals
    //

    updateReceived: qt_signal!(host_id: QString),
    monitorStateChanged: qt_signal!(host_id: QString, monitor_id: QString, new_criticality: QString),
    commandResultReceived: qt_signal!(command_result: QString, invocation_id: u64),
    monitoringDataReceived: qt_signal!(host_id: QString, category: QString, monitoring_data: QVariant, invocation_id: u64),
    errorReceived: qt_signal!(criticality: QString, error: QString),
    verificationRequested: qt_signal!(host_id: QString, connector_id: QString, message: QString, key_id: QString),

    //
    // Slots
    //

    getMonitoringData: qt_method!(fn(&self, host_id: QString, monitor_id: QString) -> QVariant),
    getDisplayData: qt_method!(fn(&self) -> QVariant),
    getCategories: qt_method!(fn(&self, host_id: QString, ignore_empty: bool) -> QStringList),
    getCategoryMonitorIds: qt_method!(fn(&self, host_id: QString, category: QString) -> QStringList),
    refresh_hosts_on_start: qt_method!(fn(&self) -> bool),
    isHostInitialized: qt_method!(fn(&self, host_id: QString) -> bool),

    getPendingMonitorCount: qt_method!(fn(&self, host_id: QString) -> u64),
    getPendingMonitorCountForCategory: qt_method!(fn(&self, host_id: QString, category: QString) -> u64),
    getPendingCommandCount: qt_method!(fn(&self, host_id: QString) -> u64),
    getPendingCommandCountForCategory: qt_method!(fn(&self, host_id: QString, category: QString) -> u64),
    getPendingCommandProgress: qt_method!(fn(&self, invocation_id: u64) -> u8),

    // These methods are used to get the data in JSON and parsed in QML side.
    // JSON is required since there doesn't seem to be a way to return a self-defined QObject.
    getSummaryMonitorData: qt_method!(fn(&self, host_id: QString) -> QStringList),
    get_host_data_json: qt_method!(fn(&self, host_id: QString) -> QString),

    //
    // Private properties
    //

    // Basically contains the state of hosts and relevant data.
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

        HostDataManagerModel {
            display_data: display_data,
            // display_options: display_options,
            display_options_category_order: priorities.into_iter().map(|(category, _)| category).collect(),
            configuration_preferences: config.preferences,
            configuration_cache_settings: config.cache_settings,
            ..Default::default()
        }
    }


    pub fn process_update(&mut self, new_display_data: frontend::HostDisplayData) {
        // HostDataModel cannot be passed between threads so parsing happens here.
        let host_state = &new_display_data.host_state;

        let maybe_old_data = self.display_data.hosts.insert(
            host_state.host.name.clone(),
            new_display_data.clone()
        );

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

        // This is the first launch so display a note about the project being in early development.
        if self.display_data.hosts.len() == 1 && self.display_data.hosts.contains_key("example-host") {
            let error = ErrorMessage::new(
                Criticality::Critical,
                String::from("Looks like this is the first time you have started LightkeeperRM.  
                            This version is still an early release and may be missing some features and contain bugs.  
                            See https://github.com/kalaksi/lightkeeper for the issue tracker and some documentation.")
            );
            self.errorReceived(QString::from(Criticality::Critical.to_string()), QString::from(error.message));
        }
    }

    pub fn reset(&mut self) {
        self.display_data.hosts.clear();
    }

    // Get monitoring data as a QVariant.
    fn getMonitoringData(&self, host_id: QString, monitor_id: QString) -> QVariant {
        let monitor_id = monitor_id.to_string();

        let display_data = self.display_data.hosts.get(&host_id.to_string()).unwrap();
        display_data.host_state.monitor_data.get(&monitor_id).unwrap().to_qvariant()
    }

    fn getDisplayData(&self) -> QVariant {
        self.display_data.to_qvariant()
    }

    // Get list of monitors for category.
    fn getCategoryMonitorIds(&self, host_id: QString, category: QString) -> QStringList {
        let display_data = self.display_data.hosts.get(&host_id.to_string()).unwrap();
        let category = category.to_string();

        let mut result = QStringList::default();

        for (monitor_id, monitor_data) in display_data.host_state.monitor_data.iter() {
            if monitor_data.display_options.category == category {
                result.push(QString::from(monitor_id.clone()));
            }
        }

        result
    }

    fn refresh_hosts_on_start(&self) -> bool {
        self.configuration_preferences.refresh_hosts_on_start
    }

    fn isHostInitialized(&self, host_id: QString) -> bool {
        if let Some(display_data) = self.display_data.hosts.get(&host_id.to_string()) {
            if self.configuration_cache_settings.prefer_cache {
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

    fn getPendingMonitorCount(&self, host_id: QString) -> u64 {
        let host_id = host_id.to_string();

        if let Some(host_display_data) = self.display_data.hosts.get(&host_id) {
            host_display_data.host_state.monitor_invocations.len() as u64
        }
        else {
            0
        }
    }

    // Currently will return only 0 or 100.
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

    /// Returns a readily sorted list of unique categories for a host. Gathered from the monitoring data.
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

    fn getSummaryMonitorData(&self, host_id: QString) -> QStringList {
        let host_id = host_id.to_string();
        let mut result = QStringList::default();

        if let Some(display_data) = self.display_data.hosts.get(&host_id) {
            let overridden_monitors = display_data.host_state.monitor_data.values()
                .filter(|data| !data.display_options.override_summary_monitor_id.is_empty())
                .map(|data| &data.display_options.override_summary_monitor_id)
                .collect::<Vec<&String>>();

            let summary_compatible = display_data.host_state.monitor_data.values()
                .filter(|data| !data.display_options.ignore_from_summary && !overridden_monitors.contains(&&data.monitor_id))
                .collect();

            let sorted_keys = self.get_monitor_data_keys_sorted(summary_compatible);

            for key in sorted_keys {
                let monitoring_data = display_data.host_state.monitor_data.get(&key).unwrap();

                result.push(serde_json::to_string(&monitoring_data).unwrap().into());
            }
        }

        result
    }

    fn get_host_data_json(&self, host_id: QString) -> QString {
        // Doesn't include monitor and command datas.
        let mut result = String::new();
        if let Some(display_data) = self.display_data.hosts.get(&host_id.to_string()) {
            let mut stripped = display_data.host_state.clone();
            stripped.monitor_data.clear();
            stripped.command_results.clear();
            result = serde_json::to_string(&stripped).unwrap();
        }

        return QString::from(result);
    }

    // TODO: remove
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
}
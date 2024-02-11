use std::collections::HashMap;
use std::thread;
use std::sync::mpsc;

extern crate qmetaobject;
use qmetaobject::*;

use crate::configuration;
use crate::enums::Criticality;
use crate::frontend;
use crate::module::monitoring::MonitoringData;
use crate::utils::ErrorMessage;


// TODO: use camelcase with qml models?
#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct HostDataManagerModel {
    base: qt_base_class!(trait QObject),

    receive_updates: qt_method!(fn(&self)),
    reset: qt_method!(fn(&mut self)),
    stop: qt_method!(fn(&mut self)),
    update_received: qt_signal!(host_id: QString),

    host_initialized: qt_signal!(host_id: QString),
    host_initialized_from_cache: qt_signal!(host_id: QString),
    monitor_state_changed: qt_signal!(host_id: QString, monitor_id: QString, new_criticality: QString),
    command_result_received: qt_signal!(command_result: QString),
    monitoring_data_received: qt_signal!(host_id: QString, category: QString, monitoring_data: QVariant),
    error_received: qt_signal!(criticality: QString, error: QString),

    get_monitoring_data: qt_method!(fn(&self, host_id: QString, monitor_id: QString) -> QVariant),
    getDisplayData: qt_method!(fn(&self) -> QVariant),
    get_categories: qt_method!(fn(&self, host_id: QString, ignore_empty: bool) -> QStringList),
    get_category_monitor_ids: qt_method!(fn(&self, host_id: QString, category: QString) -> QStringList),
    refresh_hosts_on_start: qt_method!(fn(&self) -> bool),
    is_host_initialized: qt_method!(fn(&self, host_id: QString) -> bool),

    get_pending_monitor_progress: qt_method!(fn(&self, host_id: QString) -> i8),
    get_category_pending_monitor_progress: qt_method!(fn(&self, host_id: QString, category: QString) -> i8),
    add_pending_monitor_invocations: qt_method!(fn(&self, host_id: QString, monitor_id: QString, invocation_ids: QVariantList)),
    clear_pending_monitor_invocations: qt_method!(fn(&self, host_id: QString, monitor_id: QString)),

    // These methods are used to get the data in JSON and parsed in QML side.
    // JSON is required since there doesn't seem to be a way to return a self-defined QObject.
    get_monitor_data: qt_method!(fn(&self, host_id: QString, monitor_id: QString) -> QString),
    get_summary_monitor_data: qt_method!(fn(&self, host_id: QString) -> QStringList),
    get_host_data_json: qt_method!(fn(&self, host_id: QString) -> QString),

    // Basically contains the state of hosts and relevant data. Received from HostManager.
    display_data: frontend::DisplayData,
    display_options_category_order: Vec<String>,
    /// Contains invocation IDs. Keeps track of monitoring data refresh progress. Empty when all is done.
    /// First key is host id, second key is category id. Value is a list of invocation IDs and the number of maximum pending invocations.
    pending_monitor_invocations: HashMap<String, HashMap<String, (Vec<u64>, usize)>>,
    configuration_preferences: configuration::Preferences,
    configuration_cache_settings: configuration::CacheSettings,
    update_receiver: Option<mpsc::Receiver<frontend::HostDisplayData>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,
    update_sender_prototype: Option<mpsc::Sender<frontend::HostDisplayData>>,
}

#[allow(non_snake_case)]
impl HostDataManagerModel {
    pub fn new(display_data: frontend::DisplayData, config: configuration::Configuration) -> Self {
        let mut priorities = config.display_options.as_ref().unwrap().categories.iter()
                                                                     .map(|(category, options)| (category.clone(), options.priority))
                                                                     .collect::<Vec<_>>();

        priorities.sort_by(|left, right| left.1.cmp(&right.1));

        let (sender, receiver) = mpsc::channel::<frontend::HostDisplayData>();

        let model = HostDataManagerModel {
            display_data: display_data,
            update_receiver: Some(receiver),
            update_receiver_thread: None,
            // display_options: display_options,
            display_options_category_order: priorities.into_iter().map(|(category, _)| category).collect(),
            configuration_preferences: config.preferences,
            configuration_cache_settings: config.cache_settings,
            update_sender_prototype: Some(sender),
            ..Default::default()
        };

        model
    }

    fn receive_updates(&mut self) {
        // Shouldn't (and can't) be run more than once.
        if self.update_receiver_thread.is_none() {
            let self_ptr = QPointer::from(&*self);

            let set_data = qmetaobject::queued_callback(move |new_display_data: frontend::HostDisplayData| {
                if let Some(self_pinned) = self_ptr.as_pinned() {
                    // HostDataModel cannot be passed between threads so parsing happens here.
                    let host_state = &new_display_data.host_state;

                    let maybe_old_data = self_pinned.borrow_mut().display_data.hosts.insert(
                        host_state.host.name.clone(),
                        new_display_data.clone()
                    );

                    if host_state.just_initialized {
                        ::log::debug!("Host {} initialized", host_state.host.name);
                        self_pinned.borrow().host_initialized(QString::from(host_state.host.name.clone()));
                    }
                    else if host_state.just_initialized_from_cache {
                        ::log::debug!("Host {} initialized from cache", host_state.host.name);
                        self_pinned.borrow().host_initialized_from_cache(QString::from(host_state.host.name.clone()));
                    }

                    if let Some(command_result) = new_display_data.new_command_results {
                        let json = QString::from(serde_json::to_string(&command_result).unwrap());
                        self_pinned.borrow().command_result_received(json);
                    }

                    if let Some(new_monitor_data) = new_display_data.new_monitoring_data {
                        let last_data_point = new_monitor_data.values.back().unwrap();

                        // Invocation ID may be missing if no command was executed due to error.
                        if last_data_point.invocation_id > 0 {
                            self_pinned.borrow_mut().remove_pending_monitor_invocation(&host_state.host.name,
                                                                                       &new_monitor_data.display_options.category,
                                                                                       last_data_point.invocation_id);
                        }

                        self_pinned.borrow().monitoring_data_received(QString::from(host_state.host.name.clone()),
                                                                      QString::from(new_monitor_data.display_options.category.clone()),
                                                                      new_monitor_data.to_qvariant());


                        // Find out any monitor state changes and signal accordingly.
                        if let Some(old_data) = maybe_old_data {
                            let new_criticality = new_monitor_data.values.back().unwrap().criticality;

                            if let Some(old_monitor_data) = old_data.host_state.monitor_data.get(&new_monitor_data.monitor_id) {
                                let old_criticality = old_monitor_data.values.back().unwrap().criticality;

                                if new_criticality != old_criticality {
                                    self_pinned.borrow().monitor_state_changed(
                                        QString::from(host_state.host.name.clone()),
                                        QString::from(new_monitor_data.monitor_id.clone()),
                                        QString::from(new_criticality.to_string())
                                    );
                                }
                            }
                        }
                    }

                    for error in new_display_data.new_errors {
                        self_pinned.borrow().error_received(QString::from(error.criticality.to_string()), QString::from(error.message));
                    }

                    self_pinned.borrow().update_received(QString::from(host_state.host.name.clone()));
                }
            });

            // This is the first launch so display an error about needing to edit the configuration files.
            if self.display_data.hosts.len() == 1 && self.display_data.hosts.contains_key("example-host") {
                let error = ErrorMessage::new(
                    Criticality::Critical,
                    String::from("Looks like this is the first time you have started LightkeeperRM.  
                                This version is still an early release and may be missing some features and contain bugs.  
                                See https://github.com/kalaksi/lightkeeper for the issue tracker and some documentation.")
                );
                self.error_received(QString::from(Criticality::Critical.to_string()), QString::from(error.message));
            }

            let receiver = self.update_receiver.take().unwrap();
            let thread = std::thread::spawn(move || {
                loop {
                    match receiver.recv() {
                        Ok(received_data) => {
                            if received_data.stop {
                                ::log::debug!("Gracefully exiting UI state receiver thread");
                                return;
                            }
                            else {
                                set_data(received_data);
                            }
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
    }

    pub fn new_update_sender(&self) -> mpsc::Sender<frontend::HostDisplayData> {
        self.update_sender_prototype.as_ref().unwrap().clone()
    }

    pub fn reset(&mut self) {
        self.display_data.hosts.clear();
        self.pending_monitor_invocations.clear();
    }

    pub fn stop(&mut self) {
        self.new_update_sender()
            .send(frontend::HostDisplayData::stop()).unwrap();

        if let Some(thread) = self.update_receiver_thread.take() {
            thread.join().unwrap();
        }
    }

    // TODO: remove
    fn get_monitor_data(&self, host_id: QString, monitor_id: QString) -> QString {
        if let Some(display_data) = self.display_data.hosts.get(&host_id.to_string()) {
            if let Some(monitoring_data) = display_data.host_state.monitor_data.get(&monitor_id.to_string()) {
                return QString::from(serde_json::to_string(monitoring_data).unwrap())
            }
        }
        QString::from("{}")
    }

    // Get monitoring data as a QVariant.
    fn get_monitoring_data(&self, host_id: QString, monitor_id: QString) -> QVariant {
        let monitor_id = monitor_id.to_string();

        let display_data = self.display_data.hosts.get(&host_id.to_string()).unwrap();
        display_data.host_state.monitor_data.get(&monitor_id).unwrap().to_qvariant()
    }

    fn getDisplayData(&self) -> QVariant {
        self.display_data.to_qvariant()
    }

    // Get list of monitors for category.
    fn get_category_monitor_ids(&self, host_id: QString, category: QString) -> QStringList {
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

    fn is_host_initialized(&self, host_id: QString) -> bool {
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

    fn get_pending_monitor_progress(&self, host_id: QString) -> i8 {
        let host_id = host_id.to_string();

        if let Some(categories) = self.pending_monitor_invocations.get(&host_id) {
            let max_invocations = categories.values().map(|(_, max_invocations)| max_invocations).sum::<usize>();
            let invocation_id_count = categories.values().map(|(invocation_ids, _)| invocation_ids.len()).sum::<usize>();

            if max_invocations > 0 {
                return 100 - ((invocation_id_count as f32 / max_invocations as f32 * 100.0).floor() as i8);
            }
        }

        100
    }

    fn get_category_pending_monitor_progress(&self, host_id: QString, category: QString) -> i8 {
        let host_id = host_id.to_string();
        let category = category.to_string();

        if let Some(categories) = self.pending_monitor_invocations.get(&host_id) {
            if let Some((invocation_ids, max_invocations)) = categories.get(&category) {
                if *max_invocations > 0 {
                    return 100 - ((invocation_ids.len() as f32 / *max_invocations as f32 * 100.0).floor() as i8);
                }
            }
        }

        100
    }

    fn add_pending_monitor_invocations(&mut self, host_id: QString, category: QString, invocation_ids: QVariantList) {
        let host_id = host_id.to_string();
        let category = category.to_string();
        let invocation_ids = invocation_ids.into_iter().map(|id| u64::from_qvariant(id.clone()).unwrap()).collect::<Vec<u64>>();

        let categories = self.pending_monitor_invocations.entry(host_id.clone()).or_insert_with(HashMap::new);
        let (existing_invocation_ids, max_invocations) = categories.entry(category.clone()).or_insert((Vec::new(), 0));

        *max_invocations += invocation_ids.len();
        existing_invocation_ids.extend(invocation_ids);
    }

    fn clear_pending_monitor_invocations(&mut self, host_id: QString, category: QString) {
        let host_id = host_id.to_string();
        let category = category.to_string();

        if let Some(categories) = self.pending_monitor_invocations.get_mut(&host_id) {
            if let Some((invocation_ids, max_invocations)) = categories.get_mut(&category) {
                invocation_ids.clear();
                *max_invocations = 0;
            }
        }
    }

    fn remove_pending_monitor_invocation(&mut self, host_id: &String, category: &String, invocation_id: u64) {
        if let Some(categories) = self.pending_monitor_invocations.get_mut(host_id) {
            if let Some((invocation_ids, max_invocations)) = categories.get_mut(category) {
                invocation_ids.retain(|id| *id != invocation_id);

                if invocation_ids.is_empty() {
                    *max_invocations = 0;
                }
            }
        }
        else {
            ::log::warn!("[{}] Trying to remove nonexisting monitor invocation ID {}", host_id, invocation_id);
        }
    }

    fn is_empty_category(&self, host_id: String, category: String) -> bool {
        let display_data = self.display_data.hosts.get(&host_id).unwrap();
        display_data.host_state.monitor_data.values()
            .filter(|monitor_data| monitor_data.display_options.category == category)
            .all(|monitor_data| monitor_data.values.iter()
                .all(|data_point| data_point.criticality == Criticality::Ignore || data_point.is_empty())
            )
    }

    // Get a readily sorted list of unique categories for a host. Gathered from the monitoring data.
    fn get_categories(&self, host_id: QString, ignore_empty: bool) -> QStringList {
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
            if categories.contains(prioritized_category) {
                if !ignore_empty || !self.is_empty_category(host_id.to_string(), prioritized_category.to_string()) {
                    result.push(QString::from(prioritized_category.clone()));
                }
            }
        }

        for remaining_category in categories.iter() {
            if !self.display_options_category_order.contains(remaining_category) {
                if !ignore_empty || !self.is_empty_category(host_id.to_string(), remaining_category.to_string()) {
                    result.push(QString::from(remaining_category.clone()));
                }
            }
        }

        result
    }

    fn get_summary_monitor_data(&self, host_id: QString) -> QStringList {
        let mut result = QStringList::default();
        if let Some(display_data) = self.display_data.hosts.get(&host_id.to_string()) {
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
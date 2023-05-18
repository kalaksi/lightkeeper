use std::collections::HashSet;
use std::thread;
use std::sync::mpsc;

extern crate qmetaobject;
use qmetaobject::*;

use crate::configuration;
use crate::enums::Criticality;
use crate::frontend;
use crate::module::monitoring::MonitoringData;


// TODO: use camelcase with qml models?
#[derive(QObject, Default)]
pub struct HostDataManagerModel {
    base: qt_base_class!(trait QObject),

    receive_updates: qt_method!(fn(&self)),
    update_received: qt_signal!(host_id: QString),

    host_platform_initialized: qt_signal!(host_id: QString),
    host_initialized: qt_signal!(host_id: QString, refresh_monitors: bool),
    monitor_state_changed: qt_signal!(host_id: QString, monitor_id: QString, new_criticality: QString),
    command_result_received: qt_signal!(command_result: QString),

    monitoring_data_received: qt_signal!(invocation_id: u64, category: QString, monitoring_data: QVariant),

    get_monitoring_data: qt_method!(fn(&self, host_id: QString, monitor_id: QString) -> QVariant),
    get_display_data: qt_method!(fn(&self) -> QVariant),
    is_empty_category: qt_method!(fn(&self, host_id: QString, category: QString) -> bool),
    get_categories: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_category_monitor_ids: qt_method!(fn(&self, host_id: QString, category: QString) -> QVariantList),
    refresh_hosts_on_start: qt_method!(fn(&self) -> bool),
    is_host_initialized: qt_method!(fn(&self, host_id: QString) -> bool),

    // These methods are used to get the data in JSON and parsed in QML side.
    get_monitor_data: qt_method!(fn(&self, host_id: QString, monitor_id: QString) -> QString),
    get_summary_monitor_data: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_host_data_json: qt_method!(fn(&self, host_id: QString) -> QString),

    // Basically contains the state of hosts and relevant data. Received from HostManager.
    display_data: frontend::DisplayData,
    display_options_category_order: Vec<String>,
    i_refresh_hosts_on_start: bool,
    i_bypass_cache: bool,
    update_receiver: Option<mpsc::Receiver<frontend::HostDisplayData>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,
}

impl HostDataManagerModel {
    pub fn new(display_data: frontend::DisplayData, config: configuration::Configuration) -> (Self, mpsc::Sender<frontend::HostDisplayData>) {
        let mut priorities = config.display_options.categories.iter()
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
            i_refresh_hosts_on_start: config.preferences.refresh_hosts_on_start,
            i_bypass_cache: config.cache_settings.bypass_cache,
            ..Default::default()
        };

        (model, sender)
    }

    fn receive_updates(&mut self) {
        // Shouldn't (and can't) be run more than once.
        if self.update_receiver_thread.is_none() {
            let self_ptr = QPointer::from(&*self);

            let set_data = qmetaobject::queued_callback(move |new_display_data: frontend::HostDisplayData| {
                self_ptr.as_pinned().map(|self_pinned| {
                    // HostDataModel cannot be passed between threads so parsing happens here.

                    // Update host data.
                    // There should always be old data.
                    let old_data = self_pinned.borrow_mut().display_data.hosts.insert(new_display_data.name.clone(), new_display_data.clone()).unwrap();

                    // If the platform data was unset before, it means that the host platform info was just initialized.
                    if old_data.platform.is_unset() && !new_display_data.platform.is_unset() {
                        self_pinned.borrow().host_platform_initialized(QString::from(old_data.name.clone()));
                    }

                    if !old_data.is_initialized && new_display_data.is_initialized {
                        ::log::debug!("Host {} initialized", new_display_data.name);
                        let self_borrowed = self_pinned.borrow();
                        self_borrowed.host_initialized(QString::from(old_data.name.clone()), self_borrowed.i_bypass_cache);
                    }

                    if let Some(command_result) = new_display_data.new_command_results {
                        let json = QString::from(serde_json::to_string(&command_result).unwrap());
                        self_pinned.borrow().command_result_received(json);
                    }

                    if let Some(new_monitor_data) = new_display_data.new_monitoring_data {
                        let last_data_point = new_monitor_data.values.back().unwrap();
                        self_pinned.borrow().monitoring_data_received(last_data_point.invocation_id,
                                                                      QString::from(new_monitor_data.display_options.category.clone()),
                                                                      new_monitor_data.to_qvariant());

                        // Find out any monitor state changes and signal accordingly.
                        let new_criticality = new_monitor_data.values.back().unwrap().criticality;

                        if let Some(old_monitor_data) = old_data.monitoring_data.get(&new_monitor_data.monitor_id) {
                            let old_criticality = old_monitor_data.values.back().unwrap().criticality;

                            if new_criticality != old_criticality {
                                self_pinned.borrow().monitor_state_changed(
                                    QString::from(new_display_data.name.clone()),
                                    QString::from(new_monitor_data.monitor_id.clone()),
                                    QString::from(new_criticality.to_string())
                                );
                            }
                        }
                        else {
                            self_pinned.borrow().monitor_state_changed(
                                QString::from(new_display_data.name.clone()),
                                QString::from(new_monitor_data.monitor_id.clone()),
                                QString::from(new_criticality.to_string())
                            );
                        }
                    }

                    self_pinned.borrow().update_received(QString::from(old_data.name));
                });
            });

            let receiver = self.update_receiver.take().unwrap();
            let thread = std::thread::spawn(move || {
                loop {
                    let received_data = receiver.recv().unwrap();

                    if received_data.exit_thread {
                        ::log::debug!("Gracefully exiting UI state receiver thread");
                        return;
                    }
                    set_data(received_data);
                }
            });

            self.update_receiver_thread = Some(thread);
        }
    }

    // TODO: remove
    fn get_monitor_data(&self, host_id: QString, monitor_id: QString) -> QString {
        if let Some(host) = self.display_data.hosts.get(&host_id.to_string()) {
            if let Some(monitoring_data) = host.monitoring_data.get(&monitor_id.to_string()) {
                return QString::from(serde_json::to_string(monitoring_data).unwrap())
            }
        }
        QString::from("{}")
    }

    // Get monitoring data as a QVariant.
    fn get_monitoring_data(&self, host_id: QString, monitor_id: QString) -> QVariant {
        let monitor_id = monitor_id.to_string();

        let host = self.display_data.hosts.get(&host_id.to_string()).unwrap();
        host.monitoring_data.get(&monitor_id).unwrap().to_qvariant()
    }

    fn get_display_data(&self) -> QVariant {
        self.display_data.to_qvariant()
    }

    // Get list of monitors for category.
    fn get_category_monitor_ids(&self, host_id: QString, category: QString) -> QVariantList {
        let host = self.display_data.hosts.get(&host_id.to_string()).unwrap();
        let category = category.to_string();

        let mut result = QVariantList::default();

        for (monitor_id, monitor_data) in host.monitoring_data.iter() {
            if monitor_data.display_options.category == category {
                result.push(monitor_id.to_qvariant());
            }
        }

        result
    }

    fn refresh_hosts_on_start(&self) -> bool {
        self.i_refresh_hosts_on_start
    }

    fn is_host_initialized(&self, host_id: QString) -> bool {
        if let Some(host) = self.display_data.hosts.get(&host_id.to_string()) {
            host.is_initialized
        }
        else {
            false
        }
    }

    fn is_empty_category(&self, host_id: QString, category: QString) -> bool {
        let host = self.display_data.hosts.get(&host_id.to_string()).unwrap();
        let category = category.to_string();

        host.monitoring_data.values()
                            .filter(|monitor_data| monitor_data.display_options.category == category)
                            .all(|monitor_data| monitor_data.values.iter().all(|data_point| data_point.criticality == Criticality::Ignore ||
                                                                                            data_point.is_empty()))
    }

    // Get a readily sorted list of unique categories for a host. Gathered from the monitoring data.
    fn get_categories(&self, host_id: QString) -> QVariantList {
        let host = self.display_data.hosts.get(&host_id.to_string()).unwrap();

        // Get unique categories from monitoring datas, and sort them according to config and alphabetically.
        let mut categories = host.monitoring_data.values().map(|monitor_data| monitor_data.display_options.category.clone())
                                                 .collect::<HashSet<String>>()
                                                 .into_iter().collect::<Vec<String>>();
        categories.sort();

        let mut result = QVariantList::default();

        // First add categories in the order they are defined in the config.
        for prioritized_category in self.display_options_category_order.iter() {
            if categories.contains(prioritized_category) {
                result.push(prioritized_category.to_qvariant());
            }
        }

        for remaining_category in categories.iter() {
            if !self.display_options_category_order.contains(remaining_category) {
                result.push(remaining_category.to_qvariant());
            }
        }

        result
    }

    fn get_summary_monitor_data(&self, host_id: QString) -> QVariantList {
        let mut result = QVariantList::default();
        if let Some(host) = self.display_data.hosts.get(&host_id.to_string()) {
            let summary_compatible = host.monitoring_data.values().filter(|data| !data.display_options.ignore_from_summary).collect();
            let sorted_keys = self.get_monitor_data_keys_sorted(summary_compatible);

            for key in sorted_keys {
                let monitoring_data = host.monitoring_data.get(&key).unwrap();
                result.push(serde_json::to_string(&monitoring_data).unwrap().to_qvariant())
            }
        }
        result
    }

    fn get_host_data_json(&self, host_id: QString) -> QString {
        // Doesn't include monitor and command datas.
        let mut result = String::new();
        if let Some(host_data) = self.display_data.hosts.get(&host_id.to_string()) {
            let mut stripped = host_data.clone();
            stripped.monitoring_data.clear();
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
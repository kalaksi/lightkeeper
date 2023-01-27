use std::thread;
use std::sync::mpsc;

extern crate qmetaobject;
use qmetaobject::*;

use crate::configuration;
use crate::frontend;
use crate::module::monitoring::MonitoringData;


// TODO: use camelcase with qml models?
#[derive(QObject, Default)]
pub struct HostDataManagerModel {
    base: qt_base_class!(trait QObject),

    receive_updates: qt_method!(fn(&self)),
    update_received: qt_signal!(host_id: QString),

    host_platform_initialized: qt_signal!(host_id: QString),
    monitor_state_changed: qt_signal!(host_id: QString, monitor_id: QString, new_criticality: QString),
    command_result_received: qt_signal!(command_result: QString),

    // NOTE: Couldn't get custom types to work for return types,
    // so for now methods are used to get the data in JSON and parsed in QML side.
    get_monitor_data: qt_method!(fn(&self, host_id: QString, monitor_id: QString) -> QString),
    get_monitor_datas: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_summary_monitor_data: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_host_data_json: qt_method!(fn(&self, host_id: QString) -> QString),

    display_data: frontend::DisplayData,
    // display_options: configuration::DisplayOptions,
    display_options_category_order: Vec<String>,
    update_receiver: Option<mpsc::Receiver<frontend::HostDisplayData>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,
}

impl HostDataManagerModel {
    pub fn new(display_data: frontend::DisplayData, display_options: configuration::DisplayOptions) -> (Self, mpsc::Sender<frontend::HostDisplayData>) {
        let mut priorities = display_options.categories.iter().map(|(category, options)| (category.clone(), options.priority)).collect::<Vec<_>>();
        priorities.sort_by(|left, right| left.1.cmp(&right.1));

        let (sender, receiver) = mpsc::channel::<frontend::HostDisplayData>();

        let model = HostDataManagerModel {
            display_data: display_data,
            update_receiver: Some(receiver),
            update_receiver_thread: None,
            // display_options: display_options,
            display_options_category_order: priorities.into_iter().map(|(category, _)| category).collect(),
            ..Default::default()
        };

        (model, sender)
    }

    fn receive_updates(&mut self) {
        // Shouldn't (and can't) be run more than once.
        if self.update_receiver_thread.is_none() {
            let self_ptr = QPointer::from(&*self);

            let set_data = qmetaobject::queued_callback(move |host_display_data: frontend::HostDisplayData| {
                self_ptr.as_pinned().map(|self_pinned| {
                    // HostDataModel cannot be passed between threads so parsing happens here.

                    let old_data = std::mem::replace(
                        self_pinned.borrow_mut().display_data.hosts.get_mut(&host_display_data.name).unwrap(),
                        host_display_data.clone(),
                    );

                    if old_data.platform.is_unset() && !host_display_data.platform.is_unset() {
                        self_pinned.borrow().host_platform_initialized(QString::from(old_data.name.clone()));
                    }

                    for command_result in host_display_data.command_results.values() {
                        let json = QString::from(serde_json::to_string(command_result).unwrap());
                        self_pinned.borrow().command_result_received(json);
                    }

                    // Find out any monitor state changes and signal accordingly.
                    for (monitor_id, new_monitor_data) in host_display_data.monitoring_data.iter() {
                        let new_criticality = new_monitor_data.values.last().unwrap().criticality;

                        match old_data.monitoring_data.get(monitor_id) {
                            Some(old_monitor_data) => {
                                let old_criticality = old_monitor_data.values.last().unwrap().criticality;

                                if new_criticality != old_criticality {
                                    self_pinned.borrow().monitor_state_changed(
                                        QString::from(host_display_data.name.clone()),
                                        QString::from(monitor_id.clone()),
                                        QString::from(new_criticality.to_string())
                                    );
                                }
                            },
                            None => self_pinned.borrow().monitor_state_changed(
                                QString::from(host_display_data.name.clone()),
                                QString::from(monitor_id.clone()),
                                QString::from(new_criticality.to_string())
                            ),
                        }
                    }

                    self_pinned.borrow().update_received(QString::from(old_data.name));
                });
            });

            let receiver = self.update_receiver.take().unwrap();
            let thread = std::thread::spawn(move || {
                loop {
                    let received_data = receiver.recv().unwrap();
                    set_data(received_data);
                }
            });

            self.update_receiver_thread = Some(thread);
        }
    }

    fn get_monitor_data(&self, host_id: QString, monitor_id: QString) -> QString {
        if let Some(host) = self.display_data.hosts.get(&host_id.to_string()) {
            if let Some(monitoring_data) = host.monitoring_data.get(&monitor_id.to_string()) {
                return QString::from(serde_json::to_string(monitoring_data).unwrap())
            }
        }
        QString::from("{}")
    }

    // Returns list of MonitorData structs in JSON. Empty if host doesn't exist.
    fn get_monitor_datas(&self, host_id: QString) -> QVariantList {
        let mut result = QVariantList::default();
        if let Some(host) = self.display_data.hosts.get(&host_id.to_string()) {
            let sorted_keys = self.get_monitor_data_keys_sorted(host.monitoring_data.values().collect());

            for key in sorted_keys {
                let monitoring_data = host.monitoring_data.get(&key).unwrap();
                result.push(serde_json::to_string(&monitoring_data).unwrap().to_qvariant())
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

    // Returns list of MonitorData structs in JSON. Empty if host doesn't exist.
    fn get_monitor_data_keys_sorted(&self, monitoring_data: Vec<&MonitoringData>) -> Vec<String> {
        let mut keys_ordered = Vec::<String>::new();

        // First include data of categories in an order that's defined in configuration.
        for category in self.display_options_category_order.iter() {
            let category_monitors = monitoring_data.iter().filter(|data| &data.display_options.category == category)
                                                          .collect::<Vec<&&MonitoringData>>();
            keys_ordered.extend(self.sort_by_value_type(category_monitors));
        }

        let rest_of_monitors = monitoring_data.iter().filter(|data| !keys_ordered.contains(&data.monitor_id))
                                                     .collect::<Vec<&&MonitoringData>>();
        keys_ordered.extend(self.sort_by_value_type(rest_of_monitors));

        keys_ordered
    }

    // Sorts first by value type (multivalue vs. single value) and then alphabetically.
    fn sort_by_value_type(&self, datas: Vec<&&MonitoringData>) -> Vec<String> {
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
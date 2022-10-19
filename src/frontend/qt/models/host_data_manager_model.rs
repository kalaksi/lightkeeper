use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;

extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;
use crate::module::monitoring::MonitoringData;
use super::HostDataModel;


// TODO: use camelcase with qml models?
#[derive(QObject, Default)]
pub struct HostDataManagerModel {
    base: qt_base_class!(trait QObject),

    receive_updates: qt_method!(fn(&self)),
    update_received: qt_signal!(host_id: QString),

    monitor_state_changed: qt_signal!(host_id: QString, monitor_id: QString, new_criticality: QString),
    command_result_received: qt_signal!(command_result: QString),

    // NOTE: Couldn't get custom types to work for return types,
    // so for now methods are used to get the data in JSON and parsed in QML side.
    get_monitor_data: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_monitor_data_map: qt_method!(fn(&self, host_id: QString) -> QVariantMap),
    get_host_data: qt_method!(fn(&self, host_id: QString) -> QVariantMap),

    display_data: frontend::DisplayData,
    update_receiver: Option<mpsc::Receiver<frontend::HostDisplayData>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,
}

impl HostDataManagerModel {
    pub fn new(display_data: frontend::DisplayData) -> (Self, mpsc::Sender<frontend::HostDisplayData>) {
        let (sender, receiver) = mpsc::channel::<frontend::HostDisplayData>();
        let mut model = HostDataManagerModel {
            display_data: display_data,
            update_receiver: Some(receiver),
            update_receiver_thread: None,
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

                    for (_, command_result) in &host_display_data.command_results{
                        let json = QString::from(serde_json::to_string(command_result).unwrap());
                        self_pinned.borrow().command_result_received(json);
                    }

                    // Find out any monitor state changes and signal accordingly.
                    for (monitor_id, new_monitor_data) in &host_display_data.monitoring_data {
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

    // Returns list of MonitorData structs in JSON. Empty if host doesn't exist.
    fn get_monitor_data(&self, host_id: QString) -> QVariantList {

        let mut result = HashMap::<String, HostDataModel>::new();
        for (host_id, host_data) in self.display_data.hosts.iter() {
            result.insert(host_id.clone(), HostDataModel::from(&host_data));
        }

        let mut result = QVariantList::default();
        if let Some(host) = self.display_data.hosts.get(&host_id.to_string()) {

            let mut all_monitoring_data = host.monitoring_data.clone();

            // First include data in defined order, then include rest unordered.
            for monitor_id in self.display_data.category_order.iter() {
                if let Some(monitoring_data) = all_monitoring_data.remove(monitor_id) {
                    result.push(serde_json::to_string(&monitoring_data).unwrap().to_qvariant());
                }
            }

            for monitoring_data in all_monitoring_data.values() {
                result.push(serde_json::to_string(&monitoring_data).unwrap().to_qvariant())
            }
        }

        result
    }

    // Returns map of MonitorData structs in JSON with monitor id as key. Empty if host doesn't exist.
    fn get_monitor_data_map(&self, host_id: QString) -> QVariantMap {
        let mut result = QVariantMap::default();
        // TODO: how to deal with sorting

        if let Some(host) = self.display_data.hosts.get(&host_id.to_string()) {
            for (monitor_id, monitor_data) in &host.monitoring_data {
                result.insert(
                    QString::from(monitor_id.clone()),
                    serde_json::to_string(&monitor_data).unwrap().to_qvariant()
                );
            }
        }

        result
    }

    fn get_host_data(&self, host_id: QString) -> QVariantMap {
        let mut result = QVariantMap::default();
    
        if let Some(host_data) = self.display_data.hosts.get(&host_id.to_string()) {
            // TODO: use table headers from display data in init? Or remove table headers completely.
            result.insert(QString::from("Status"), QString::from(host_data.status.to_string()).to_qvariant());
            result.insert(QString::from("Name"), QString::from(host_data.name.clone()).to_qvariant());
            result.insert(QString::from("FQDN"), QString::from(host_data.domain_name.to_string()).to_qvariant());
            result.insert(QString::from("IP"), QString::from(host_data.ip_address.to_string()).to_qvariant());
        }
    
        result
    }
}
use std::collections::HashMap;
use std::sync::mpsc;
use crate::Host;
use crate::module::monitoring::{MonitoringModule, DataPoint};
use crate::connection_manager::ConnectorMessage;

pub struct MonitorManager {
    // Host as the first key, monitor name as the second.
    host_monitors: HashMap<Host, HashMap<String, (mpsc::Sender<ConnectorMessage>, Box<dyn MonitoringModule>)>>,
}

impl MonitorManager {
    pub fn new() -> Self {
        MonitorManager {
            host_monitors: HashMap::new(),
        }
    }

    // Adds a monitor but only if a monitor with the same ID doesn't exist.
    pub fn add_monitor(&mut self, host: &Host, monitor: Box<dyn MonitoringModule>, sender: mpsc::Sender<ConnectorMessage>) {
        loop {
            if let Some(monitors) = self.host_monitors.get_mut(&host) {
                let module_spec = monitor.get_module_spec();

                if let None = monitors.get_mut(&module_spec.id) {
                    log::debug!("Adding monitor {}", module_spec.id);

                    monitors.insert(module_spec.id, (sender, monitor));
                }

                break;
            }
            else {
                self.host_monitors.insert(host.clone(), HashMap::new());
            }
        }
    }

    pub fn start(&mut self) {
        for (host, monitors) in self.host_monitors.iter() {
            log::info!("Refreshing monitoring data for host {}", host.name);

            for (monitor_id, (sender, monitor)) in monitors.iter() {

                sender.send(ConnectorMessage {
                    destination: host.clone(),
                    connector_id: monitor_id.clone(),
                    payload: monitor.get_connector_message(),
                });
            }

        }
    }
}
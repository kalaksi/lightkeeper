use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::sync::{Arc, Mutex};

use crate::Host;
use crate::module::monitoring::Monitor;
use crate::host_manager::DataPointMessage;
use crate::connection_manager::{ ConnectorRequest, ConnectorResponse };

// Monitor id is the key.
type MonitorCollection = HashMap<String, MessageHandler>;

pub struct MonitorManager {
    monitors: Arc<Mutex<HashMap<Host, MonitorCollection>>>,
    response_sender_prototype: mpsc::Sender<ConnectorResponse>,
    receiver_handle: Option<thread::JoinHandle<()>>,
}

impl MonitorManager {
    pub fn new(state_update_channel: Sender<DataPointMessage>) -> Self {
        let (sender, receiver) = mpsc::channel::<ConnectorResponse>();
        let monitors = Arc::new(Mutex::new(HashMap::new()));

        let handle = Self::process(monitors.clone(), receiver, state_update_channel.clone());

        MonitorManager {
            monitors: monitors,
            response_sender_prototype: sender,
            receiver_handle: Some(handle),
        }
    }

    // Adds a monitor but only if a monitor with the same ID doesn't exist.
    pub fn add_monitor(&mut self, host: &Host, monitor: Monitor, sender: mpsc::Sender<ConnectorRequest>) {
        loop {
            let mut monitors = self.monitors.lock().unwrap();

            if let Some(monitor_handlers) = monitors.get_mut(&host) {
                let module_spec = monitor.get_module_spec();

                if let None = monitor_handlers.get_mut(&module_spec.id) {
                    log::debug!("Adding monitor {}", module_spec.id);

                    monitor_handlers.insert(module_spec.id, MessageHandler {
                        monitor: monitor,
                        connector_channel: Mutex::new(sender),
                    });
                }

                break;
            }
            else {
                monitors.insert(host.clone(), HashMap::new());
            }
        }
    }

    pub fn join(&mut self) {
        self.receiver_handle.take().expect("Thread has already stopped.")
                            .join().unwrap();
    }

    pub fn refresh_monitors(&self) {
        Self::process_refresh_monitors(self.monitors.clone(), self.response_sender_prototype.clone());
    }

    fn process_refresh_monitors(
        monitors: Arc<Mutex<HashMap<Host, MonitorCollection>>>,
        response_sender_prototype: mpsc::Sender<ConnectorResponse>
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let monitors = monitors.lock().unwrap();

            for (host, monitor_handlers) in monitors.iter() {
                log::info!("Refreshing monitoring data for host {}", host.name);

                for (monitor_id, monitor_handler) in monitor_handlers.iter() {
                    let connector_channel = monitor_handler.connector_channel.lock().unwrap();

                    connector_channel.send(ConnectorRequest {
                        connector_id: monitor_handler.monitor.get_connector_spec().id,
                        monitor_id: monitor_id.clone(),
                        host: host.clone(),
                        message: monitor_handler.monitor.get_connector_message(),
                        response_channel: response_sender_prototype.clone(),
                    }).unwrap_or_else(|error| {
                        log::error!("Couldn't send message to connector: {}", error);
                    });
                }
            }
        })
    }

    fn process(
        monitors: Arc<Mutex<HashMap<Host, MonitorCollection>>>,
        receiver: mpsc::Receiver<ConnectorResponse>,
        state_update_channel: Sender<DataPointMessage>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
                let response = receiver.recv().unwrap();

                let monitors = monitors.lock().unwrap();
                if let Some(host_monitors) = monitors.get(&response.host) {
                    if let Some(handler) = host_monitors.get(&response.monitor_id) {

                        match handler.monitor.process_response(&response.host, &response.message) {
                            Ok(data_point) => {
                                log::debug!("Data point received: {}", data_point);
                                state_update_channel.send(DataPointMessage {
                                    host_name: response.host.name,
                                    display_options: handler.monitor.get_display_options(),
                                    module_spec: handler.monitor.get_module_spec(),
                                    data_point: data_point
                                }).unwrap_or_else(|error| {
                                    log::error!("Couldn't send message to state manager: {}", error);
                                });
                            },
                            Err(error) => {
                                log::error!("Error from monitor: {}", error);
                            }
                        }

                    }
                    else {
                        log::error!("Host monitor {} does not exist.", response.monitor_id);
                    }
                }
                else {
                    log::error!("Host {} does not exist.", response.host.name);
                }
            }
        })
    }
}

struct MessageHandler {
    monitor: Monitor,
    connector_channel: Mutex<Sender<ConnectorRequest>>,
}

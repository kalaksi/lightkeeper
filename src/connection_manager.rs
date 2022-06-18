use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::Host;
use crate::module::connection::{ ConnectionModule, Connector };

type ConnectorCollection = HashMap<String, Box<dyn ConnectionModule + Send>>;

pub struct ConnectionManager {
    // Collection of ConnectionModules that can be shared between threads.
    // Host as the first hashmap key, connector id as the second.
    connectors: Arc<Mutex<HashMap<Host, ConnectorCollection>>>,
    request_sender_prototype: mpsc::Sender<ConnectorRequest>,
    receiver_handle: Option<thread::JoinHandle<()>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<ConnectorRequest>();
        let connectors = Arc::new(Mutex::new(HashMap::new()));

        let handle = Self::start_receiving_messages(connectors.clone(), receiver);

        ConnectionManager {
            connectors: connectors,
            request_sender_prototype: sender,
            receiver_handle: Some(handle),
        }
    }

    // Adds a connector but only if a connector with the same ID doesn't exist.
    // This call will block if process_messages() is currently handling a message.
    pub fn add_connector(&mut self, host: &Host, connector: Connector) -> mpsc::Sender<ConnectorRequest> {
        loop {
            let mut connectors = self.connectors.lock().unwrap();

            if let Some(host_connections) = connectors.get_mut(&host) {
                let module_spec = connector.get_module_spec();

                if let None = host_connections.get_mut(&module_spec.id) {
                    host_connections.insert(module_spec.id, connector);
                }

                return self.request_sender_prototype.clone();
            }
            else {
                connectors.insert(host.clone(), HashMap::new());
            }
        }
    }

    pub fn join(&mut self) {
        self.receiver_handle.take().expect("Thread has already stopped.")
                            .join().unwrap();
    }

    fn start_receiving_messages(
        connectors: Arc<Mutex<HashMap<Host, ConnectorCollection>>>,
        receiver: mpsc::Receiver<ConnectorRequest>
    ) -> thread::JoinHandle<()> {

        thread::spawn(move || {
            loop {
                let request = receiver.recv().unwrap();

                log::debug!("Connector message received: {}", request.connector_id);

                let mut connectors = connectors.lock().unwrap();
                let connector = connectors.get_mut(&request.host)
                                          .and_then(|connections| connections.get_mut(&request.connector_id)).unwrap();

                let mut connector_is_connected = false;
                let output = match connector.connect(&request.host.ip_address) {
                    Ok(()) => {
                        connector_is_connected = true;

                        connector.send_message(&request.message).unwrap_or_else(|error| {
                            log::error!("Error while refreshing monitoring data: {}", error);

                            // Double check the connection status
                            connector_is_connected = connector.is_connected();
                            String::from("")
                        })
                    },
                    Err(error) => {
                        log::error!("Error while connecting: {}", error);
                        String::from("")
                    }
                };

                let response = ConnectorResponse {
                    connector_id: request.connector_id,
                    monitor_id: request.monitor_id,
                    host: request.host,
                    message: output,
                    connector_is_connected: connector_is_connected,
                };

                if let Err(error) = request.response_channel.send(response) {
                    log::error!("Failed to send response from connector: {}", error);
                }
            }
        })
    }

}

pub struct ConnectorRequest {
    pub connector_id: String,
    pub monitor_id: String,
    pub host: Host,
    pub message: String,
    pub response_channel: mpsc::Sender<ConnectorResponse>,
}

pub struct ConnectorResponse {
    pub connector_id: String,
    pub monitor_id: String,
    pub host: Host,
    pub message: String,
    pub connector_is_connected: bool,
}
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::Host;
use crate::module::{
    Module,
    connection::Empty,
    connection::ConnectionModule,
    monitoring::DataPoint,
};

pub struct ConnectionManager {
    // Collection of ConnectionModules that can be shared between threads.
    // Host as the first hashmap key, connector id as the second.
    connectors: Arc<Mutex<HashMap<Host, HashMap<String, Box<dyn ConnectionModule + Send>>>>>,
    message_sender: mpsc::Sender<ConnectorMessage>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<ConnectorMessage>();

        let connection_manager = ConnectionManager {
            connectors: Arc::new(Mutex::new(HashMap::new())),
            message_sender: sender,
        };

        Self::process_messages(connection_manager.connectors.clone(), receiver);

        connection_manager
    }

    // Adds a connector but only if a connector with the same ID doesn't exist.
    // This call will block if process_messages() is currently handling a message.
    pub fn add_connector(&mut self, host: &Host, connector: Box<dyn ConnectionModule + Send>) -> mpsc::Sender<ConnectorMessage> {
        loop {
            let mut connectors = self.connectors.lock().unwrap();

            if let Some(host_connections) = connectors.get_mut(&host) {
                let module_spec = connector.get_module_spec();

                if let None = host_connections.get_mut(&module_spec.id) {
                    host_connections.insert(module_spec.id, connector);
                }

                return self.message_sender.clone();
            }
            else {
                connectors.insert(host.clone(), HashMap::new());
            }
        }
    }


    fn process_messages(
        connectors: Arc<Mutex<HashMap<Host, HashMap<String, Box<dyn ConnectionModule + Send>>>>>,
        receiver: mpsc::Receiver<ConnectorMessage>
    ) {
        thread::spawn(move || {
            loop {
                let message = receiver.recv().unwrap();

                let mut connectors = connectors.lock().unwrap();
                let connector = connectors.get_mut(&message.destination)
                                          .and_then(|connections| connections.get_mut(&message.connector_id)).unwrap();

                if let Err(error) = connector.connect(&message.destination.ip_address) {
                    log::error!("Error while connecting: {}", error);
                }

                let response = connector.send_message(&message.payload);

                // send for processing?

                let new_data = response.unwrap_or_else(|error| {
                    log::error!("Error while refreshing monitoring data: {}", error);
                    String::new()
                    // DataPoint::empty_and_critical()
                });
            }
        });
    }

}

pub struct ConnectorMessage {
    pub destination: Host,
    pub connector_id: String,
    pub payload: String,
}
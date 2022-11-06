use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::Host;
use crate::module::connection::{
    ConnectionModule,
    Connector,
    ResponseMessage,
};

pub type ResponseHandlerCallback = Box<dyn FnOnce(Vec<Result<ResponseMessage, String>>, bool) + Send + 'static>;
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
    pub fn add_connector(&mut self, host: &Host, connector: Connector) {
        loop {
            let mut connectors = self.connectors.lock().unwrap();

            if let Some(host_connections) = connectors.get_mut(&host) {
                let module_spec = connector.get_module_spec();

                if let None = host_connections.get_mut(&module_spec.id) {
                    host_connections.insert(module_spec.id, connector);
                }

                return;
            }
            else {
                connectors.insert(host.clone(), HashMap::new());
            }
        }
    }

    pub fn new_request_sender(&mut self) -> mpsc::Sender<ConnectorRequest> {
        return self.request_sender_prototype.clone()
    }

    pub fn join(&mut self) {
        self.receiver_handle.take().expect("Thread has already stopped.")
                            .join().unwrap();
    }

    fn start_receiving_messages(
        connectors: Arc<Mutex<HashMap<Host, ConnectorCollection>>>,
        receiver: mpsc::Receiver<ConnectorRequest>
    ) -> thread::JoinHandle<()> {

        // TODO: threadpool
        thread::spawn(move || {
            loop {
                let request = receiver.recv().unwrap();

                // Not normally enabled as this may log passwords.
                // log::debug!("Connector request received for {}: {}", request.connector_id, request.message);

                let mut connectors = connectors.lock().unwrap();
                let connector = connectors.get_mut(&request.host)
                                          .and_then(|connections| connections.get_mut(&request.connector_id)).unwrap();

                let mut connector_is_connected = false;

                match connector.connect(&request.host.ip_address) {
                    Ok(()) => connector_is_connected = true,
                    Err(error) => log::error!("error while connecting {}: {}", request.host.ip_address, error),
                }

                let mut responses = Vec::<Result<ResponseMessage, String>>::new();
                if connector_is_connected {
                    for message in &request.messages {
                        log::debug!("sending message: {}", message);

                        match connector.send_message(message) {
                            Ok(response) => {
                                let return_code = response.return_code;
                                responses.push(Ok(response));

                                if return_code != 0 {
                                    // Don't continue if any of the commands fail unexpectedly.
                                    break;
                                }
                            }
                            Err(error) => {
                                log::error!("error while sending data: {}", error);

                                // Double check the connection status.
                                connector_is_connected = connector.is_connected();
                                responses.push(Err(error));
                                break;
                            }
                        };
                    }
                }

                (request.response_handler)(responses, connector_is_connected);
            }
        })
    }

}

pub struct ConnectorRequest {
    pub connector_id: String,
    pub source_id: String,
    pub host: Host,
    pub messages: Vec<String>,
    pub response_handler: ResponseHandlerCallback,
}
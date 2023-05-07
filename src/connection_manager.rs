use std::fmt::Debug;
use std::{
    collections::HashMap,
    sync::mpsc,
    sync::Arc,
    sync::Mutex,
    thread,
};

use rayon;
use rayon::prelude::*;

use crate::Host;
use crate::file_handler;
use crate::module::ModuleFactory;
use crate::module::ModuleSpecification;
use crate::module::connection::*;

pub type ResponseHandlerCallback = Box<dyn FnOnce(Vec<Result<ResponseMessage, String>>) + Send + 'static>;
type ConnectorStates = HashMap<ModuleSpecification, Arc<Mutex<Connector>>>;

const MAX_WORKER_THREADS: usize = 4;

pub struct ConnectionManager {
    stateful_connectors: Option<HashMap<String, ConnectorStates>>,
    request_receiver: Option<mpsc::Receiver<ConnectorRequest>>,
    request_sender_prototype: mpsc::Sender<ConnectorRequest>,
    receiver_thread: Option<thread::JoinHandle<()>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<ConnectorRequest>();

        ConnectionManager {
            stateful_connectors: Some(HashMap::new()),
            request_receiver: Some(receiver),
            request_sender_prototype: sender,
            receiver_thread: None,
        }
    }

    // Adds a connector but only if a connector with the same ID doesn't exist.
    pub fn add_connector(&mut self, host: &Host, connector: Connector) {
        let connectors = self.stateful_connectors.as_mut().unwrap();
        connectors.entry(host.name.clone()).or_insert(HashMap::new());

        if let Some(host_connectors) = connectors.get_mut(&host.name) {
            let module_spec = connector.get_module_spec();

            if !host_connectors.contains_key(&module_spec) {
                log::debug!("[{}] Adding connector {}", host.name, module_spec.id);
                host_connectors.insert(module_spec, Arc::new(Mutex::new(connector)));
            }
        }
    }

    pub fn new_request_sender(&mut self) -> mpsc::Sender<ConnectorRequest> {
        return self.request_sender_prototype.clone()
    }

    pub fn start(&mut self, module_factory: ModuleFactory) {
        let thread = Self::start_receiving_requests(
            self.stateful_connectors.take().unwrap(),
            self.request_receiver.take().unwrap(),
            module_factory
        );
        self.receiver_thread = Some(thread);
    }

    pub fn join(&mut self) {
        self.receiver_thread.take().expect("Thread has already stopped.")
                            .join().unwrap();
    }

    fn start_receiving_requests(
        mut stateful_connectors: HashMap<String, ConnectorStates>,
        receiver: mpsc::Receiver<ConnectorRequest>,
        module_factory: ModuleFactory
    ) -> thread::JoinHandle<()> {

        let worker_pool = rayon::ThreadPoolBuilder::new().num_threads(MAX_WORKER_THREADS).build().unwrap();
        log::debug!("Created worker pool with {} threads", MAX_WORKER_THREADS);

        thread::spawn(move || {
            loop {
                let request = match receiver.recv() {
                    Ok(data) => data,
                    Err(error) => {
                        log::error!("Stopped receiver thread: {}", error);
                        return;
                    }
                };

                if request.request_type == RequestType::Exit {
                    log::debug!("Gracefully exiting connection manager thread");
                    return;
                }

                // Requests with no connector dependency.
                if request.connector_spec.is_none() {
                    (request.response_handler)(Vec::new());
                }
                else {
                    let connector_spec = request.connector_spec.as_ref().unwrap().clone();
                    let connector_metadata = module_factory.get_connector_module_metadata(&connector_spec);
                    let request_messages = request.messages.clone();

                    // Stateless connectors.
                    if connector_metadata.is_stateless {
                        // Wrapping to Arc<Mutex<>> to allow passing to Self::process_request.
                        let mutex_request = Arc::new(Mutex::new(request));

                        worker_pool.install(|| {
                            let responses = request_messages.par_iter().map(|request_message| {
                                log::debug!("Worker {} processing stateless request {}", rayon::current_thread_index().unwrap(), request_message);
                                let connector = module_factory.new_connector(&connector_spec, &HashMap::new());
                                Self::process_request(mutex_request.clone(), &request_message, Arc::new(Mutex::new(connector)))
                            }).collect();

                            let request = Arc::try_unwrap(mutex_request).unwrap().into_inner().unwrap();
                            (request.response_handler)(responses);
                        });
                    }
                    // Stateful connectors.
                    else {
                        // TODO: This will block the thread unnecessarily. Need better solution. Imagine
                        // MAX_WORKERS amount of requests sequentially for the same host and connector.
                        let connector_mutex = stateful_connectors.get_mut(&request.host.name)
                                                                 .and_then(|connections| connections.get_mut(&connector_spec)).unwrap();

                        let mut connector = connector_mutex.lock().unwrap();
                        if !connector.is_connected() {
                            if let Err(error) = connector.connect(&request.host.ip_address) {
                                log::error!("[{}] Error while connecting {}: {}", request.host.name, request.host.ip_address, error);
                                // TODO: put the request data somewhere? Now gets dropped.
                                continue;
                            }
                        }
                        drop(connector);

                        let request_mutex = Arc::new(Mutex::new(request));
                        worker_pool.install(|| {
                            let responses = request_messages.iter().map(|request_message| {
                                log::debug!("Worker {} processing stateful request {}", rayon::current_thread_index().unwrap(), request_message);
                                Self::process_request(request_mutex.clone(), &request_message, connector_mutex.clone())
                            }).collect();

                            let request = Arc::try_unwrap(request_mutex).unwrap().into_inner().unwrap();
                            (request.response_handler)(responses);
                        });
                    }
                }
            }
        })
    }


    fn process_request(request: Arc<Mutex<ConnectorRequest>>, request_message: &String, connector: Arc<Mutex<Connector>>) -> Result<ResponseMessage, String> {
        let request = request.lock().unwrap();
        let mut connector = connector.lock().unwrap();

        match &request.request_type {
            RequestType::Command => {
                log::debug!("[{}] Processing command: {}", request.host.name, request_message);
                let response_result = connector.send_message(&request_message);
                    // Don't continue if any of the commands fail unexpectedly.
                if response_result.is_ok() && response_result.as_ref().unwrap().return_code != 0 {
                    Err(String::from("Command returned non-zero exit code"))
                }
                else {
                    response_result
                }
            },
            RequestType::Download => {
                log::debug!("[{}] Downloading file: {}", request.host.name, request_message);
                match connector.download_file(&request_message) {
                    Ok(contents) => {
                        match file_handler::create_file(&request.host, &request_message, contents) {
                            Ok(file_path) => Ok(ResponseMessage::new(file_path)),
                            Err(error) => Err(error.to_string()),
                        }
                    },
                    Err(error) => Err(error.to_string()),
                }
            },
            RequestType::Upload => {
                log::debug!("[{}] Uploading file: {}", request.host.name, request_message);
                match file_handler::read_file(&request_message) {
                    Ok((metadata, contents)) => {
                        let mut result = connector.upload_file(&metadata.remote_path, contents);
                        if result.is_ok() && metadata.temporary {
                            log::debug!("removing temporary local file");
                            result = file_handler::remove_file(&request_message);
                        }

                        if result.is_ok() {
                            Ok(ResponseMessage::empty())
                        }
                        else {
                            Err(result.unwrap_err().to_string())
                        }
                    },
                    Err(error) => Err(error.to_string()),
                }
            },
            RequestType::Exit => panic!(),
        }
    }
}

pub struct ConnectorRequest {
    pub connector_spec: Option<ModuleSpecification>,
    pub source_id: String,
    pub host: Host,
    pub messages: Vec<String>,
    pub request_type: RequestType,
    pub response_handler: ResponseHandlerCallback,
}

impl ConnectorRequest {
    pub fn exit_token() -> Self {
        ConnectorRequest {
            connector_spec: None,
            source_id: String::new(),
            host: Host::new(&String::new(), &String::from("127.0.0.1"), &String::new(), &Vec::new()).unwrap(),
            messages: Vec::new(),
            request_type: RequestType::Exit,
            response_handler: Box::new(|_| ()),
        }
    }
}

impl Debug for ConnectorRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ConnectorRequest {:?}]", self.connector_spec)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestType {
    Command,
    Download,
    Upload,
    Exit,
}
use std::sync::mpsc;
use std::io::{self, BufRead, Read, Write};
use std::time::{Duration, SystemTime};
use std::{process, thread};


use crate::error::LkError;
use crate::{file_handler, utils};
use crate::frontend::UIUpdate;
use std::process::Command;
use crate::metrics::tmserver::{self, RequestType, TMSRequest, TMSResponse};

//
// NOTE: This is MetrcisManager that handles connections to metrics database for keeping record of host metrics for graphs.
// Only TMServer is currently supported. It is a simple, lightweight, locally run, closed-source metrics database server and is developed by the Lightkeeper project.
// It is tailored for the needs of Lightkeeper, but is independent and can be used with any software.
//
// TMServer is a closed-source binary and requires a license (trial licenses are available) to make an open-core model possible.
// My dream is to dedicate more time to open source software, but it's hard without any financial support, since I'll also have to work full-time.
//
// Using TMServer and metrics is optional and the binary is not installed by default. It is downloaded from GitHub on demand and verified.
// Even though it's closed-source, the communication protocol is open (see tmserver/tmsrequest.rs).
// The metrics server does not use or need network access (it uses unix sockets) and can't send malicious input to Lightkeeper.
//



/// In milliseconds.
const SERVICE_EXIT_WAIT_TIME: u64 = 5000;

pub struct MetricsManager {
    process_handle: process::Child,
    log_thread: Option<thread::JoinHandle<()>>,
    request_thread: Option<thread::JoinHandle<()>>,

    /// Every request gets an invocation ID. Valid numbers begin from 1.
    invocation_id_counter: u64,
    // TODO: support remote database backends in the future.
    request_sender: mpsc::Sender<TMSRequest>,
}


impl MetricsManager {
    pub fn new(update_sender: mpsc::Sender<UIUpdate>) -> Result<Self, LkError> {
        let (process_handle, log_thread) = Self::start_service()?;
        let (request_sender, request_receiver) = mpsc::channel();

        let request_thread = thread::spawn(move || {
            Self::process_requests(request_receiver, update_sender.clone());
        });

        Ok(MetricsManager {
            process_handle: process_handle,
            log_thread: Some(log_thread),
            request_thread: Some(request_thread),
            invocation_id_counter: 1,
            request_sender: request_sender,
        })
    }

    /// Downloads (if needed) and verifies Pro Services binary and then spawns a new process for it.
    fn start_service() -> io::Result<(process::Child, thread::JoinHandle<()>)> {
        log::info!("Starting Lightkeeper Pro service");
        let pro_services_path = file_handler::get_cache_dir()?.join("lightkeeper-pro-services");
        let signature_path = pro_services_path.with_extension("sig");

        // TODO: Add license check.
        // The binary is not included by default so download it first.
        if let Err(_) = std::fs::metadata(&pro_services_path) {
            // TODO: actual paths
            utils::download::download_file("https://raw.githubusercontent.com/kalaksi/lightkeeper/develop/README.md", pro_services_path.to_str().unwrap())?;
            utils::download::download_file("https://raw.githubusercontent.com/kalaksi/lightkeeper/develop/README.md.sig", signature_path.to_str().unwrap())?;
        }

        // Don't verify when developing.
        let do_verification = !cfg!(debug_assertions);
        if do_verification {
            let sign_cert = include_bytes!("../../certs/sign.crt");
            utils::download::verify_signature(&pro_services_path, &signature_path, sign_cert)?;
        }

        // Start Lightkeeper Pro Services process. Failure is not critical, but some features will be unavailable.
        let mut process_handle = Command::new(pro_services_path)
            // Logs are printed to stderr by default.
            .stderr(process::Stdio::piped())
            .spawn()?;

        let log_thread;

        if let Some(stderr) = process_handle.stderr.take() {
            let stderr_reader = std::io::BufReader::new(stderr);
            log_thread = thread::spawn(move || {
                for line in stderr_reader.lines() {
                    match line {
                        Ok(line) => log::info!("{}", line),
                        Err(error) => {
                            log::error!("Error while reading process output: {}", error);
                        }
                    }
                }
            });
        }
        else {
            return Err(io::Error::new(io::ErrorKind::Other, "Couldn't capture process output."));
        }

        // Wait a little bit for server to start.
        thread::sleep(Duration::from_millis(100));

        Ok((process_handle, log_thread))
    }


    pub fn stop(&mut self) -> Result<(), LkError> {
        let service_request = TMSRequest::exit();
        let _ignored = self.request_sender.clone().send(service_request);

        let mut waited = 0;
        loop {
            if let Ok(_) = self.process_handle.try_wait() {
                break;
            }

            if waited >= SERVICE_EXIT_WAIT_TIME {
                log::warn!("Forcing Lightkeeper Pro service to stop.");
                if let Err(error) = self.process_handle.kill() {
                    log::error!("Failed to kill process: {}", error);
                }

                break;
            }

            thread::sleep(Duration::from_millis(100));
            waited += 100;
        }

        if let Some(request_thread) = self.request_thread.take() {
            if request_thread.join().is_err() {
                log::error!("Error while waiting for request thread");
            }
        }

        if let Some(log_thread) = self.log_thread.take() {
            if log_thread.join().is_err() {
                log::error!("Error while waiting for log thread");
            }
        }

        Ok(())
    }

    pub fn insert_metrics(&mut self, host_id: &str, monitor_id: &str, metrics: &[super::Metric]) -> Result<u64, LkError> {
        let invocation_id = self.send_request(RequestType::MetricsInsert {
            host_id: host_id.to_string(),
            metric_id: monitor_id.to_string(),
            metrics: metrics.iter().map(|metric| tmserver::Metric::from(metric.clone())).collect(),
        })?;

        Ok(invocation_id)
    }

    pub fn get_metrics(&mut self, host_id: &str, monitor_id: &str, start_time: i64, end_time: i64) -> Result<u64, LkError> {
        let invocation_id = self.send_request(RequestType::MetricsQuery {
            host_id: host_id.to_string(),
            metric_id: monitor_id.to_string(),
            start_time: start_time,
            end_time: end_time,
        })?;

        Ok(invocation_id)
    }


    fn send_request(&mut self, request_type: RequestType) -> Result<u64, LkError> {
        let invocation_id = self.invocation_id_counter;
        self.invocation_id_counter += 1;

        let current_unix_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u32;

        let service_request = TMSRequest {
            request_id: invocation_id,
            time: current_unix_ms,
            request_type: request_type,
        };

        self.request_sender.clone().send(service_request)
                                   .map_err(|error| LkError::other(format!("Failed to send request: {}", error)))?;
        Ok(invocation_id)
    }

    fn process_requests(request_receiver: mpsc::Receiver<TMSRequest>, update_sender: mpsc::Sender<UIUpdate>) {
        let data_dir = file_handler::get_cache_dir().unwrap();
        let mut tls_stream = match tmserver::setup_connection(&data_dir) {
            Ok(stream) => stream,
            Err(error) => {
                log::error!("Failed to connect to Pro Services: {}", error);
                return;
            }
        };

        loop {
            // These should never fail.
            let service_request = request_receiver.recv().unwrap();
            let serialized = bincode::serialize(&service_request).unwrap();

            // TODO: send errors to UI?
            if let Err(error) = tls_stream.write_all(&serialized) {
                log::error!("Failed to send request: {}", error);

                match service_request.request_type {
                    RequestType::Exit => break,
                    _ => continue,
                };
            }

            if let Err(error) = tls_stream.flush() {
                log::error!("Failed to send request: {}", error);

                match service_request.request_type {
                    RequestType::Exit => break,
                    _ => continue,
                };
            }

            let mut buffer = vec![0; 1024];

            let read_count = match service_request.request_type {
                RequestType::Exit => {
                    match tls_stream.read_to_end(&mut buffer) {
                        Ok(count) => count,
                        Err(error) => {
                            log::error!("Failed to read response: {}", error);
                            break;
                        }
                    }
                },
                _ => {
                    match tls_stream.read(&mut buffer) {
                        Ok(count) => count,
                        Err(error) => {
                            log::error!("Failed to read response: {}", error);
                            continue;
                        }
                    }
                }
            };

            if read_count == 0 {
                log::error!("No data received.");
            }

            let response = match bincode::deserialize::<TMSResponse>(&buffer) {
                Ok(response) => response,
                Err(error) => {
                    log::error!("Failed to deserialize response: {}", error);
                    continue;
                }
            };

            log::debug!("Request took {} ms", response.lag);

            if response.errors.len() > 0 {
                log::error!("Service error: {}", response.errors.join(". "));
            }

            match service_request.request_type {
                RequestType::Exit => break,
                _ => (),
            };

            if let Err(error) = update_sender.send(UIUpdate::Chart(response)) {
                log::error!("Failed to send update: {}", error);
            }
        }
    }
}
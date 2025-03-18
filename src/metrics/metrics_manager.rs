use std::io::{self, BufRead, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};
use std::{process, thread};

use crate::error::LkError;
use crate::frontend::UIUpdate;
use crate::metrics::lmserver::{self, RequestType, LMSRequest, LMSResponse};
use crate::file_handler;

//
// NOTE: This is MetricsManager that handles connections to metrics server that stores host metrics for charts.
// Only LMServer (LightMetricsServer) is currently supported. It is a simple, lightweight, locally run, closed-source metrics database server and is developed by the Lightkeeper project.
// It is tailored for the needs of Lightkeeper, but is independent and could be used with any software.
//
// LMServer is closed-source to help make an open-core model possible.
//
// Using LMServer and metrics is optional. The binary is not installed by default but is downloaded automatically from GitLab (where it's built and signed) and verified.
// The communication protocol is open, see lmserver/lmsrequest.rs.
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
    request_sender: mpsc::Sender<LMSRequest>,
}

impl MetricsManager {
    pub fn new(update_sender: mpsc::Sender<UIUpdate>) -> Result<Self, LkError> {
        let (process_handle, log_thread) = Self::start_service()?;
        let (request_sender, request_receiver) = mpsc::channel();

        let request_thread = thread::spawn(move || {
            if Self::process_requests(request_receiver, update_sender.clone()).is_err() {

            }
        });

        Ok(MetricsManager {
            process_handle: process_handle,
            log_thread: Some(log_thread),
            request_thread: Some(request_thread),
            invocation_id_counter: 1,
            request_sender: request_sender,
        })
    }

    /// Downloads (if needed) and verifies metrics server binary and then spawns a new process for it.
    fn start_service() -> io::Result<(process::Child, thread::JoinHandle<()>)> {
        log::info!("Starting metrics server");
        let data_dir = file_handler::get_data_dir()?;
        let lmserver_path = data_dir.join("lmserver");

        // If data directory is missing, this is probably the first run, so create data directory.
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        let signature_path = lmserver_path.with_extension("sig");
        // Check and store version info for detecting updates.
        let current_lmserver_version = "v0.1.9";
        let version_file_path = lmserver_path.with_extension("version");
        let download_lmserver = match std::fs::metadata(&version_file_path) {
            Err(_) => {
                log::debug!("Downloading Light Metrics Server");
                true
            }
            Ok(_) => {
                let version = std::fs::read_to_string(&version_file_path)?;
                if version.trim() != current_lmserver_version {
                    log::debug!("New version of Light Metrics Server available");
                    true
                }
                else {
                    false
                }
            }
        };

        // The binary is not included by default so download it first.
        if download_lmserver {
            use base64::{Engine as _, engine::general_purpose};

            // Simple read-only token is used to try to limit metrics server downloads to Lightkeeper.
            let token_b64 = download_string("https://github.com/kalaksi/lightkeeper/raw/refs/heads/master/src/metrics/download-token.txt")?;
            // let token_b64 = "".chars().zip("".bytes()).map(|(b, k)| (b as u8) ^ k).collect::<Vec<u8>>();
            let token = general_purpose::STANDARD.decode(token_b64.as_str())
                .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?
                .iter()
                .zip("ktn86rdoktc26bwv431o4whcno".bytes())
                .map(|(b, k)| (b ^ k) as char)
                .collect::<String>();

            // Built and signed with GitLab CI.
            download_file(
                &format!("https://gitlab.com/api/v4/projects/68049585/repository/files/lmserver.sig/raw?ref={}", current_lmserver_version),
                &token,
                signature_path.to_str().unwrap(),
            )?;
            download_file(
                &format!("https://gitlab.com/api/v4/projects/68049585/repository/files/lmserver/raw?ref={}", current_lmserver_version),
                &token,
                lmserver_path.to_str().unwrap(),
            )?;

            // Update version info.
            std::fs::write(&version_file_path, current_lmserver_version)?;

            // Make sure lmserver is executable.
            std::fs::set_permissions(&lmserver_path, std::fs::Permissions::from_mode(0o755))?;
        }

        let sign_cert = include_bytes!("../../certs/sign.crt");
        verify_signature(&lmserver_path, &signature_path, sign_cert)?;

        // Start Light Metrics Server process. Failure is not critical, but some features will be unavailable.
        let mut process_handle = Command::new(lmserver_path)
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
        // TODO: replace with some kind of signalling.
        thread::sleep(Duration::from_millis(100));

        Ok((process_handle, log_thread))
    }

    pub fn stop(&mut self) -> Result<(), LkError> {
        let service_request = LMSRequest::exit();
        if self.request_sender.clone().send(service_request).is_err() {
            if let Err(error) = self.process_handle.kill() {
                log::error!("Failed to kill process: {}", error);
            }
        }

        let mut waited = 0;
        loop {
            if let Ok(_) = self.process_handle.try_wait() {
                break;
            }

            if waited >= SERVICE_EXIT_WAIT_TIME {
                log::warn!("Forcing metrics server to stop.");
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
            metrics: metrics.iter().map(|metric| lmserver::Metric::from(metric.clone())).collect(),
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

        let service_request = LMSRequest {
            request_id: invocation_id,
            time: current_unix_ms,
            request_type: request_type,
        };

        self.request_sender
            .clone()
            .send(service_request)
            .map_err(|error| LkError::other(format!("Failed to send request: {}", error)))?;
        Ok(invocation_id)
    }

    fn process_requests(request_receiver: mpsc::Receiver<LMSRequest>, update_sender: mpsc::Sender<UIUpdate>) -> Result<(), ()> {
        let data_dir = file_handler::get_data_dir().unwrap();
        let mut tls_stream = match lmserver::setup_connection(&data_dir) {
            Ok(stream) => stream,
            Err(error) => {
                log::error!("Failed to connect to metrics server: {}", error);
                return Err(());
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
                RequestType::Exit => match tls_stream.read_to_end(&mut buffer) {
                    Ok(count) => count,
                    Err(error) => {
                        log::error!("Failed to read response: {}", error);
                        break;
                    }
                },
                _ => match tls_stream.read(&mut buffer) {
                    Ok(count) => count,
                    Err(error) => {
                        log::error!("Failed to read response: {}", error);
                        continue;
                    }
                },
            };

            if read_count == 0 {
                log::error!("No data received.");
            }

            let response = match bincode::deserialize::<LMSResponse>(&buffer) {
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

        Ok(())
    }
}
/// Function to download a file using ureq.
fn download_file(url: &str, access_token: &str, output_path: &str) -> io::Result<()> {
    let response = ureq::get(url)
        .set("PRIVATE-TOKEN", access_token)
        // Keep timeout short so it doesn't block startup too long in case there's no access to internet.
        .timeout(std::time::Duration::from_secs(2))
        .call()
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

    if response.status() == 200 {
        let mut file = std::fs::File::create(output_path)?;
        let mut reader = response.into_reader();
        io::copy(&mut reader, &mut file)?;

        log::debug!("Downloaded file: {}", output_path);
        Ok(())
    }
    else {
        Err(io::Error::new(io::ErrorKind::Other, "Failed to download file.")) 
    }
}

fn download_string(url: &str) -> io::Result<String> {
    let response = ureq::get(url)
        // Keep timeout short so it doesn't block startup too long in case there's no access to internet.
        .timeout(std::time::Duration::from_secs(2))
        .call()
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

    if response.status() == 200 {
        let mut reader = response.into_reader();
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        Ok(content)
    }
    else {
        Err(io::Error::new(io::ErrorKind::Other, "Failed to download file."))
    }
}

fn verify_signature(file_path: &std::path::Path, signature_path: &std::path::Path, sign_cert: &[u8]) -> io::Result<()> {
    let file_bytes = std::fs::read(file_path)?;
    let signature = std::fs::read(signature_path)?;
    let sign_cert = openssl::x509::X509::from_pem(sign_cert)?.public_key()?;

    let mut verifier = openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), &sign_cert)?;
    verifier.update(&file_bytes)?;

    if !verifier.verify(&signature)? {
        log::error!("{} signature verification failed.", file_path.file_name().unwrap_or_default().to_string_lossy());
        Err(io::Error::new(io::ErrorKind::Other, "Signature verification failed."))
    }
    else {
        log::debug!("{} signature verified.", file_path.file_name().unwrap_or_default().to_string_lossy());
        Ok(())
    }
}

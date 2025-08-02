/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::io::{self, BufRead, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};
use std::{process, thread};

use crate::error::LkError;
use crate::frontend::UIUpdate;
use crate::metrics::lmserver::{self, RequestType, LMSRequest, LMSResponse};
use crate::file_handler;

//
// NOTE: MetricsManager handles connections to metrics server that stores host metrics for charts.
// Only LMServer (Light Metrics Server) is currently supported. It is a simple, lightweight, locally run,
// metrics server developed by the Lightkeeper project. It is closed-source to help make open-core model possible.
//
// Using LMServer and metrics is optional. The server is not installed by default.
// It's downloaded automatically if charts are enabled. Download is sourced from GitLab (where it's built and signed) and verified.
// The communication protocol is open, see lmserver/lmsrequest.rs. The protocol can not be used to send malicious input to Lightkeeper.
// The metrics server does not require network access (it uses unix sockets) and if using flatpak, this is enforced.
//

/// In milliseconds.
const SERVICE_EXIT_WAIT_TIME: u64 = 5000;

pub struct MetricsManager {
    process_handle: Option<process::Child>,
    log_thread: Option<thread::JoinHandle<()>>,
    request_thread: Option<thread::JoinHandle<()>>,
    request_sender: Option<mpsc::Sender<LMSRequest>>,

    /// Every request gets an invocation ID. Valid numbers begin from 1.
    invocation_id_counter: u64,
    // TODO: support remote database backends in the future?
    update_sender: mpsc::Sender<UIUpdate>,
}

impl MetricsManager {
    pub fn new(update_sender: mpsc::Sender<UIUpdate>) -> Self {
        MetricsManager {
            process_handle: None,
            log_thread: None,
            request_thread: None,
            request_sender: None,
            invocation_id_counter: 1,
            update_sender: update_sender,
        }
    }

    /// Downloads (if needed) and verifies metrics server binary and then spawns a new process for it.
    pub fn start_service(&mut self) -> Result<(), LkError> {
        if self.is_running() {
            return Ok(())
        }

        log::info!("Starting metrics server");
        let data_dir = file_handler::get_data_dir()?;
        let lmserver_path = data_dir.join("lmserver");
        let signature_path = lmserver_path.with_extension("sig");
        let socket_path = lmserver_path.with_extension("sock");
        let version_file_path = lmserver_path.with_extension("version");

        // There exists obvious classic race conditions regarding file handling.
        if socket_path.exists() {
            log::debug!("Found existing socket file. Trying to stop existing process.");
            let (request_sender, request_receiver) = mpsc::channel();
            let new_update_sender = self.update_sender.clone();
            self.request_sender = Some(request_sender);
            
            match lmserver::setup_connection(&socket_path) {
                Err(_error) => {
                    log::debug!("Failed to connect to metrics server, removing socket file");
                    std::fs::remove_file(&socket_path)?;
                },
                Ok(tls_stream) => {
                    Self::process_requests(tls_stream, request_receiver, new_update_sender);
                    // It's enough to only set self.request_sender before trying to stop.
                    self.stop()?;
                }
            }
        }

        // If data directory is missing, this is probably the first run, so create data directory.
        std::fs::create_dir_all(&data_dir)?;

        Self::download_lmserver(&lmserver_path, &signature_path)?;

        let sign_cert = include_bytes!("../../certs/sign.crt");
        if let Err(error) = verify_signature(&lmserver_path, &signature_path, sign_cert) {
            // Delete files to force re-download and to lower the risk of accidentally running possibly unverified binary.
            std::fs::remove_file(&lmserver_path)?;
            std::fs::remove_file(&signature_path)?;
            std::fs::remove_file(&version_file_path)?;
            return Err(LkError::from(error));
        }

        // Start Light Metrics Server process. Failure is not critical, but charts will be unavailable.
        // Logs are printed to stderr by default. With flatpak, access is further restricted.
        let is_flatpak = std::env::var("FLATPAK_ID").is_ok();
        let mut process_handle = if is_flatpak {
            Command::new("flatpak-spawn")
                .arg("--no-network")
                .arg(lmserver_path)
                .arg("-d")
                .arg(data_dir)
                .stderr(process::Stdio::piped())
                .spawn()?
        }
        else {
            Command::new(lmserver_path)
                .arg("-d")
                .arg(data_dir)
                .stderr(process::Stdio::piped())
                .spawn()?
        };


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
            return Err(LkError::other("Couldn't capture process output."));
        }

        self.process_handle = Some(process_handle);
        self.log_thread = Some(log_thread);

        // Wait a little bit for server to start.
        // TODO: replace with some kind of signalling.
        thread::sleep(Duration::from_millis(100));

        let (request_sender, request_receiver) = mpsc::channel();
        self.request_sender = Some(request_sender);

        let new_update_sender = self.update_sender.clone();

        self.request_thread = Some(thread::spawn(move || {
            match lmserver::setup_connection(&socket_path) {
                Err(error) => {
                    log::error!("Failed to connect to metrics server: {}", error);
                    // TODO: send error to UI?
                },
                Ok(tls_stream) => {
                    Self::process_requests(tls_stream, request_receiver, new_update_sender);
                }
            }
        }));

        Ok(())
    }

    fn download_lmserver(lmserver_path: &Path, signature_path: &Path) -> io::Result<()> {
        // Check and store version info for detecting updates.
        let current_lmserver_version = "v0.1.10";
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

            // Simple read-only token is used to try to limit metrics server downloads to Lightkeeper and
            // to make the repo less public. Obfuscated to keep bots away.
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

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.request_sender.is_some()
    }

    pub fn stop(&mut self) -> Result<(), LkError> {
        let mut stop_success = false;

        if let Some(request_sender) = self.request_sender.take() {
            let exit_request = LMSRequest::exit();
            if request_sender.send(exit_request).is_ok() {
                stop_success = true;
            }
        }

        if let Some(mut process_handle) = self.process_handle.take() {
            if !stop_success {
                match process_handle.kill() {
                    Ok(_) => stop_success = true,
                    Err(error) => log::error!("Failed to kill process: {}", error),
                }
            }

            let mut waited = 0;
            loop {
                if let Ok(_) = process_handle.try_wait() {
                    stop_success = true;
                    break;
                }

                if waited >= SERVICE_EXIT_WAIT_TIME {
                    log::warn!("Forcing metrics server to stop.");
                    match process_handle.kill() {
                        Ok(_) => stop_success = true,
                        Err(error) => log::error!("Failed to kill process: {}", error),
                    }

                    break;
                }

                thread::sleep(Duration::from_millis(100));
                waited += 100;
            }
        }

        // Remove socket file.
        if stop_success {
            let data_dir = file_handler::get_data_dir()?;
            let socket_path = data_dir.join("lmserver.sock");
            if socket_path.exists() {
                std::fs::remove_file(&socket_path)?;
            }
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
        if let Some(request_sender) = self.request_sender.as_ref() {
            let invocation_id = self.invocation_id_counter;
            self.invocation_id_counter += 1;

            let current_unix_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u32;

            let service_request = LMSRequest {
                request_id: invocation_id,
                time: current_unix_ms,
                request_type: request_type,
            };

            request_sender.send(service_request).map_err(|error| LkError::other(format!("Failed to send request: {}", error)))?;
            Ok(invocation_id)
        }
        else {
            return Err(LkError::other("Metrics are not available."));
        }
    }

    fn process_requests(mut tls_stream: rustls::StreamOwned<rustls::ClientConnection, UnixStream>, request_receiver: mpsc::Receiver<LMSRequest>, update_sender: mpsc::Sender<UIUpdate>) {
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

            if response.lag > 100 {
                log::debug!("Request took {} ms", response.lag);
            }
            else if response.lag > 500 {
                log::warn!("Request took {} ms", response.lag);
            }

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

use std::io::{self, BufRead, Read, Write};
use std::sync::Arc;
use std::os::unix::net::UnixStream;
use std::time::Duration;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{process, thread, path};
use openssl;


use crate::file_handler;
use crate::module::monitoring::DataPoint;

use std::process::Command;

//
// NOTE: This is a module for interfacing with Lightkeeper Pro Services extension.
// It is a closed-source binary that provides additional features to Lightkeeper to make an open-core model possible (free in beta).
// The extension is optional and the binary is not installed by default, but downloaded from GitHub on demand.
// Even though it's closed-source, the communication protocol is open (see this file), so you can verify what kind of requests are sent and received.
// The Pro Services extension does not use or need network access and can't send malicious input to Lightkeeper (according to bincode).
// The binary is built and signed using GitHub Actions.
//



const SOCKET_PATH: &str = "services.sock";
/// In milliseconds.
const CONNECTION_TIMEOUT: u64 = 10000;
/// In milliseconds.
const PRO_SERVICES_EXIT_WAIT_TIME: u64 = 5000;

#[derive(Default)]
pub struct ProService {
    tls_stream: Option<rustls::StreamOwned<rustls::ClientConnection, UnixStream>>,
    process_handle: Option<process::Child>,
    log_thread: Option<thread::JoinHandle<()>>,
}


impl ProService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Downloads (if needed) and verifies Pro Services binary and then spawns a new process for it.
    pub fn start(&mut self) -> io::Result<()> {
        // TODO: Add license check.
        // The binary is not included by default so download it first.
        // The versions go hand in hand with Lightkeeper and not updated separately.
        log::info!("Starting Lightkeeper Pro service");

        let pro_services_path = file_handler::get_cache_dir()?.join("lightkeeper-pro-services");
        let signature_path = pro_services_path.with_extension("sig");

        if let Err(_) = std::fs::metadata(&pro_services_path) {
            // TODO: actual paths
            download_file("https://raw.githubusercontent.com/kalaksi/lightkeeper/develop/README.md", pro_services_path.to_str().unwrap())?;
            download_file("https://raw.githubusercontent.com/kalaksi/lightkeeper/develop/README.md.sig", signature_path.to_str().unwrap())?;
        }

        verify_signature(&pro_services_path, &signature_path)?;
        start_service(&pro_services_path).map(|(process_handle, log_thread)| {
            self.process_handle = Some(process_handle);
            self.log_thread = Some(log_thread);
        })?;
        // Wait a little bit for server to start.
        thread::sleep(Duration::from_millis(100));
        self.tls_stream = Some(Self::setup_connection()?);
        Ok(())
    }

    pub fn is_available(&self) -> bool {
        self.tls_stream.is_some()
    }

    pub fn stop(&mut self) {
        if let Some(mut process) = self.process_handle.take() {
            let response = self.process_request(ServiceRequest::exit());

            if response.is_err() {
                log::warn!("Waiting before forcing Lightkeeper Pro service to exit...");
                thread::sleep(Duration::from_millis(PRO_SERVICES_EXIT_WAIT_TIME));
                if let Err(error) = process.kill() {
                    log::error!("Failed to kill process: {}", error);
                }
            }
        }

        if let Some(thread) = self.log_thread.take() {
            if let Err(_) = thread.join() {
                log::error!("Error while waiting for log thread");
            }
        }

        self.tls_stream = None;
    }

    pub fn process_request(&mut self, request: ServiceRequest) -> io::Result<ServiceResponse> {
        let tls_stream = match &mut self.tls_stream {
            Some(stream) => stream,
            None => return Err(io::Error::new(io::ErrorKind::Other, "Service connection is not set up.")),
        };

        let serialized = bincode::serialize(&request).map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        tls_stream.write_all(&serialized)?;

        tls_stream.flush()?;

        let mut buffer = vec![0; 1024];
        let read_count = match request.request_type {
            RequestType::Exit => tls_stream.read_to_end(&mut buffer)?,
            _ => tls_stream.read(&mut buffer)?,
        };

        if read_count == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "No data received."));
        }

        let response = bincode::deserialize::<ServiceResponse>(&buffer)
                               .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

        log::debug!("Request took {} ms", response.lag);
        
        if response.errors.len() > 0 {
            let error = format!("Pro service error: {}", response.errors.join(". "));
            Err(io::Error::new(io::ErrorKind::Other, error))
        }
        else {
            Ok(response)
        }
    }

    fn setup_connection() -> io::Result<rustls::StreamOwned<rustls::ClientConnection, UnixStream>> {
        let socket_path = file_handler::get_cache_dir()?.join(SOCKET_PATH);

        let unix_stream = match UnixStream::connect(&socket_path) {
            Ok(stream) => stream,
            Err(_) => {
                // Wait some more and try one more time.
                thread::sleep(Duration::from_millis(500));
                UnixStream::connect(&socket_path)?
            }
        };

        unix_stream.set_read_timeout(Some(Duration::from_millis(CONNECTION_TIMEOUT)))?;
        unix_stream.set_write_timeout(Some(Duration::from_millis(CONNECTION_TIMEOUT)))?;

        let tls_config = Arc::new(Self::setup_client_tls());
        let server_name = rustls::pki_types::ServerName::try_from("localhost").unwrap();
        let tls_connection = rustls::ClientConnection::new(tls_config.clone(), server_name)
                                                    .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

        Ok(rustls::StreamOwned::new(tls_connection, unix_stream))
    }

    fn setup_client_tls() -> rustls::ClientConfig {
        let ca_cert_pem = include_str!("../certs/ca.crt");

        let client_cert_pem = "
-----BEGIN CERTIFICATE-----
MIIBrDCCAVGgAwIBAgIUaLPJjErlG+MnFq3yAa+RFcrsZFcwCgYIKoZIzj0EAwIw
JTEjMCEGA1UEAwwaTGlnaHRrZWVwZXIgUHJvIExpY2Vuc2UgQ0EwHhcNMjQwOTE0
MjAyMzQwWhcNMzkwOTExMjAyMzQwWjAUMRIwEAYDVQQDDAlsb2NhbGhvc3QwWTAT
BgcqhkjOPQIBBggqhkjOPQMBBwNCAASO74MZwT54t+osf4GGmSZ6F28U8xIm57CG
IHePgfgzqvbfi3e/SOihr7Q5ViSuHOz54PqOEk3LTuPoCb2VqEPOo3AwbjAfBgNV
HSMEGDAWgBQwQ00JHMba+aeyu/uqMrxcmcpsHDAJBgNVHRMEAjAAMAsGA1UdDwQE
AwIE8DAUBgNVHREEDTALgglsb2NhbGhvc3QwHQYDVR0OBBYEFFOox6MT9F2MB+XC
C9MptUr89G8pMAoGCCqGSM49BAMCA0kAMEYCIQC2CLIoSp+xB3d3QH5Xu2Rwq9Tf
YUdOMEGbF+uJUJBJ1AIhAJG14OnE4e9iT/OgeNTYWJt57MCuiiLEUWk9ESBHF60S
-----END CERTIFICATE-----";

        // NOTE: not really private, as you can probably see.
        // Client auth doesn't really currently offer much benefit, but it was part of the original design and was left be for now.
        let client_key_pem = "
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIGKn2QiNNyVpe4rwfndGbNU4VvgkCuupLLDN+3pD4aTcoAoGCCqGSM49
AwEHoUQDQgAEju+DGcE+eLfqLH+BhpkmehdvFPMSJuewhiB3j4H4M6r234t3v0jo
oa+0OVYkrhzs+eD6jhJNy07j6Am9lahDzg==
-----END EC PRIVATE KEY-----";

        let client_key = rustls_pemfile::ec_private_keys(&mut io::Cursor::new(client_key_pem)).next().unwrap().unwrap();
        let client_cert = rustls_pemfile::certs(&mut io::Cursor::new(client_cert_pem)).next().unwrap().unwrap();
        let mut store = rustls::RootCertStore::empty();

        for result in rustls_pemfile::certs(&mut io::Cursor::new(ca_cert_pem)) {
            if let Ok(cert) = result {
                store.add(cert.clone()).unwrap();
            }
        }

        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(store)
            .with_client_auth_cert(vec![client_cert], client_key.into()).unwrap();

        tls_config
    }
}

fn verify_signature(file_path: &path::Path, signature_path: &path::Path) -> io::Result<()> {
    let file_bytes = std::fs::read(file_path)?;
    let signature = std::fs::read(signature_path)?;
    let sign_cert = openssl::x509::X509::from_pem(include_bytes!("../certs/sign.crt"))?.public_key()?;

    let mut verifier = openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), &sign_cert)?;
    verifier.update(&file_bytes)?;

    // Don't verify when developing.
    let do_verification = !cfg!(debug_assertions);

    if do_verification && !verifier.verify(&signature)?  {
        Err(io::Error::new(io::ErrorKind::Other, "Binary signature verification failed."))
    }
    else {
        Ok(())
    }
}

fn start_service(service_path: &path::Path) -> io::Result<(process::Child, thread::JoinHandle<()>)> {
    // Start Lightkeeper Pro Services process. Failure is not critical, but some features will be unavailable.
    let mut process_handle = Command::new(service_path)
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

    Ok((process_handle, log_thread))
}

// Function to download a file using ureq
fn download_file(url: &str, output_path: &str) -> io::Result<()> {
    let response = ureq::get(url).call().map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

    if response.status() == 200 {
        let mut file = std::fs::File::create(output_path)?;
        let mut reader = response.into_reader();
        io::copy(&mut reader, &mut file)?;

        log::info!("Downloaded file: {}", output_path);
    } else {
        log::error!("Failed to download file ({}): {}", response.status(), url);
    }
    Ok(())
}

// fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
//     let response = bincode::deserialize::<ServiceResponse>(&response.message.as_bytes()).unwrap();
// }

// fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
//     let request = ServiceRequest {
//         request_id: 1,
//         request_type: RequestType::MetricsInsert {
//             host_id: "test_host".to_string(),
//             monitor_id: "test_monitor".to_string(),
//             metrics: vec![Metric {
//                 label: "".to_string(),
//                 value: 23,
//                 ..Default::default()
//             }],
//         },
//     };

//     let serialized = bincode::serialize(&request).unwrap();
//     Ok(serialized)
// }

/// Do not change without updating the Pro Services extension.
#[derive(Serialize, Deserialize)]
pub struct ServiceRequest {
    pub request_id: u32,
    /// Requester sets this to unix time in milliseconds.
    pub time: u32,
    pub request_type: RequestType,
}

impl ServiceRequest {
    pub fn exit() -> Self {
        ServiceRequest {
            request_id: 0,
            time: 0,
            request_type: RequestType::Exit,
        }
    }
}

/// Do not change without updating the Pro Services extension.
#[derive(Serialize, Deserialize)]
pub enum RequestType {
    Healthcheck,
    Exit,
    MetricsInsert {
        host_id: String,
        monitor_id: String,
        metrics: Vec<Metric>,
    },
    MetricsQuery {
        host_id: String,
        /// Unix timestamp in seconds.
        start_time: i64,
        /// Unix timestamp in seconds.
        end_time: i64,
    },
}

/// Do not change without updating the Pro Services extension.
#[derive(Default, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub request_id: u32,
    // In milliseconds. 0 if not set.
    pub lag: u32,
    pub metrics: HashMap<String, Vec<Metric>>,
    pub errors: Vec<String>,
}

/// Do not change without updating the Pro Services extension.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Metric {
    pub time: i64,
    pub label: String,
    pub value: i64,
}
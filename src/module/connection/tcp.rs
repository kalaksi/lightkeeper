use base64::engine::general_purpose;
use base64::Engine;
use rustls::pki_types::{CertificateDer, ServerName};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use lightkeeper_module::connection_module;
use crate::error::LkError;
use crate::module::*;
use crate::module::connection::*;

#[connection_module(
    name="tcp",
    version="0.0.1",
    cache_scope="Global",
    description="Connects using TCP. Used for testing TCP (and TLS) connections.",
    settings={
        timeout => "How many seconds to wait for response. Default: 10.",
        verify_certificate => "Assume TLS (e.g. HTTPS) and verify certificate. Default: false.",
        ca_certificates_path => "Path to the CA certificates bundle file. Expects PEM format. Default: (empty).",
    }
)]
pub struct Tcp {
    timeout: u16,
    verify_certificate: bool,
    rustls_client_config: Option<Arc<rustls::ClientConfig>>,
}

impl Tcp {
}

impl Module for Tcp {
    fn new(settings: &HashMap<String, String>) -> Self {
        let verify_certificate: bool = settings.get("verify_certificate").map(|value| value.parse().unwrap_or_default()).unwrap_or(false);
        let ca_certificates_path = settings.get("ca_certificates_path").cloned();
        let mut store = rustls::RootCertStore::empty();

        let client_config = if verify_certificate {
            match rustls_native_certs::load_native_certs() {
                Ok(roots) => { 
                    for cert in roots {
                        if let Err(_) = store.add(cert) {
                            log::error!("Failed to add certificate to CA certificate store.");
                        }
                    }
                },
                Err(error) => log::error!("Failed to load OS CA certificates: {}", error),
            }

            if let Some(path) = ca_certificates_path.as_ref() {
                for cert in load_certs(Path::new(path)) {
                    if let Err(_) = store.add(cert) {
                        log::error!("Failed to add certificate to CA certificate store.");
                    }
                }
            }

            Some(Arc::new(rustls::ClientConfig::builder().with_root_certificates(store).with_no_client_auth()))
        }
        else {
            None
        };

        Tcp {
            verify_certificate: verify_certificate,
            timeout: settings.get("timeout").map(|value| value.parse().unwrap_or_default()).unwrap_or(10),
            rustls_client_config: client_config
        }
    }
}

impl ConnectionModule for Tcp {
    /// Connects to the specified address and returns the result.
    /// With `verify_certificate` enabled, returns the certificate chain in PEM format.
    /// With `verify_certificate` disabled, returns an empty string and uses exit code to determine success.
    fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError> {
        // Connect and verify TLS certificate.
        if self.verify_certificate {
            let rustls_address: ServerName = message.to_string().try_into().map_err(|_| LkError::other_p("Invalid address", message))?;

            match rustls::ClientConnection::new(self.rustls_client_config.clone().unwrap(), rustls_address) {
                Ok(client) => {
                    let pem_chain = client.peer_certificates().unwrap_or_default().into_iter()
                                          .map(|cert| der_to_pem(&cert))
                                          .collect::<Vec<String>>()
                                          .join("\n");

                    Ok(ResponseMessage::new_success(pem_chain))
                },
                Err(error) => Ok(ResponseMessage::new_error(format!("{}", error))),
            }
        }
        // Only connect.
        else {
            let socket_addr: std::net::SocketAddr = message.to_string().parse().map_err(|error| LkError::other_p("Invalid address", error))?;
            let result = std::net::TcpStream::connect_timeout(&socket_addr, std::time::Duration::from_secs(self.timeout as u64));

            match result {
                Ok(_) => Ok(ResponseMessage::new_success("".to_string())),
                Err(error) => Ok(ResponseMessage::new_error(format!("{}", error))),
            }
        }
    }
}

fn load_certs(path: &Path) -> Vec<CertificateDer> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => {
            log::error!("Failed to open file: {}", path.display());
            return vec![];
        }
    };

    let mut file = std::io::BufReader::new(file);

    let (certs, errors): (Vec<_>, Vec<_>) = rustls_pemfile::certs(&mut file).partition(Result::is_ok);
    let certs: Vec<_> = certs.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

    if errors.len() > 0 {
        log::error!("Failed to load some certificates from {}: {:?}", path.display(), errors);
    }

    certs
}

fn der_to_pem(der_bytes: &[u8]) -> String {
    let base64 = general_purpose::STANDARD.encode(der_bytes);
    format!("-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----", base64)
}
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{io, path};

/// In milliseconds.
const CONNECTION_TIMEOUT: u64 = 5000;

pub fn setup_connection(socket_path: &path::Path) -> io::Result<rustls::StreamOwned<rustls::ClientConnection, UnixStream>> {
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

    let ca_cert_pem = include_str!("../../../certs/ca.crt");
    let tls_config = Arc::new(setup_client_tls(ca_cert_pem, None, None));

    let server_name = rustls::pki_types::ServerName::try_from("tms").unwrap();
    let tls_connection = rustls::ClientConnection::new(tls_config.clone(), server_name)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

    Ok(rustls::StreamOwned::new(tls_connection, unix_stream))
}

fn setup_client_tls(ca_cert_pem: &str, client_cert_pem: Option<&str>, client_key_pem: Option<&str>) -> rustls::ClientConfig {
    let mut store = rustls::RootCertStore::empty();

    for result in rustls_pemfile::certs(&mut io::Cursor::new(ca_cert_pem)) {
        if let Ok(cert) = result {
            store.add(cert.clone()).unwrap();
        }
    }

    if client_cert_pem.is_some() && client_key_pem.is_some() {
        let client_cert_pem = client_cert_pem.unwrap();
        let client_key_pem = client_key_pem.unwrap();

        let client_key = rustls_pemfile::ec_private_keys(&mut io::Cursor::new(client_key_pem))
            .next()
            .unwrap()
            .unwrap();
        let client_cert = rustls_pemfile::certs(&mut io::Cursor::new(client_cert_pem)).next().unwrap().unwrap();

        rustls::ClientConfig::builder()
            .with_root_certificates(store)
            .with_client_auth_cert(vec![client_cert], client_key.into())
            .unwrap()
    }
    else {
        rustls::ClientConfig::builder()
            .with_root_certificates(store)
            .with_no_client_auth()
    }
}

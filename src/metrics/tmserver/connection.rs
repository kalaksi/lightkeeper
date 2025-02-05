use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{io, path};

/// In milliseconds.
const CONNECTION_TIMEOUT: u64 = 10000;

pub fn setup_connection(data_dir: &path::Path) -> io::Result<rustls::StreamOwned<rustls::ClientConnection, UnixStream>> {
    let socket_path = data_dir.join("tmserver.sock");

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
    let tls_config = Arc::new(setup_client_tls(ca_cert_pem));

    let server_name = rustls::pki_types::ServerName::try_from("tms").unwrap();
    let tls_connection =
        rustls::ClientConnection::new(tls_config.clone(), server_name).map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

    Ok(rustls::StreamOwned::new(tls_connection, unix_stream))
}

fn setup_client_tls(ca_cert_pem: &str) -> rustls::ClientConfig {
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

    let client_key = rustls_pemfile::ec_private_keys(&mut io::Cursor::new(client_key_pem))
        .next()
        .unwrap()
        .unwrap();
    let client_cert = rustls_pemfile::certs(&mut io::Cursor::new(client_cert_pem)).next().unwrap().unwrap();
    let mut store = rustls::RootCertStore::empty();

    for result in rustls_pemfile::certs(&mut io::Cursor::new(ca_cert_pem)) {
        if let Ok(cert) = result {
            store.add(cert.clone()).unwrap();
        }
    }

    let tls_config = rustls::ClientConfig::builder()
        .with_root_certificates(store)
        .with_client_auth_cert(vec![client_cert], client_key.into())
        .unwrap();

    tls_config
}

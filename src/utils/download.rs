use std::{io, path};

/// Function to download a file using ureq.
pub fn download_file(url: &str, output_path: &str) -> io::Result<()> {
    let response = ureq::get(url).call().map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

    if response.status() == 200 {
        let mut file = std::fs::File::create(output_path)?;
        let mut reader = response.into_reader();
        io::copy(&mut reader, &mut file)?;

        log::info!("Downloaded file: {}", output_path);
    }
    else {
        log::error!("Failed to download file ({}): {}", response.status(), url);
    }
    Ok(())
}

pub fn verify_signature(file_path: &path::Path, signature_path: &path::Path, sign_cert: &[u8]) -> io::Result<()> {
    let file_bytes = std::fs::read(file_path)?;
    let signature = std::fs::read(signature_path)?;
    let sign_cert = openssl::x509::X509::from_pem(sign_cert)?.public_key()?;

    let mut verifier = openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), &sign_cert)?;
    verifier.update(&file_bytes)?;

    // Don't verify when developing.
    let do_verification = !cfg!(debug_assertions);

    if do_verification && !verifier.verify(&signature)? {
        Err(io::Error::new(io::ErrorKind::Other, "Signature verification failed."))
    }
    else {
        Ok(())
    }
}

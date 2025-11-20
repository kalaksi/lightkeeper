/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::{env, fs, io, path::Path, path::PathBuf};

use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

use crate::file_handler;
use crate::utils::sha256;
use crate::Host;

const MAX_PATH_COMPONENTS: u8 = 2;
const APP_DIR_NAME: &str = "lightkeeper";
const METADATA_SUFFIX: &str = ".metadata.yml";

pub fn get_config_dir() -> PathBuf {
    let mut config_dir = if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(path)
    }
    else if let Some(path) = env::var_os("HOME") {
        PathBuf::from(path).join(".config")
    }
    else {
        panic!("Cannot find configuration directory. $XDG_CONFIG_HOME or $HOME is not set.");
    };

    // If not running inside flatpak, we need to add a separate subdir for the app.
    if env::var("FLATPAK_ID").is_err() {
        config_dir = config_dir.join(APP_DIR_NAME);
    }

    config_dir
}

pub fn get_cache_dir() -> PathBuf {
    let mut cache_dir = if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(path)
    }
    else if let Some(home_path) = env::var_os("HOME") {
        PathBuf::from(home_path).join(".cache")
    }
    else {
        panic!("Cannot find cache directory. $XDG_CACHE_HOME or $HOME is not set.");
    };

    // If not running inside flatpak, we need to add a separate subdir for the app.
    if env::var("FLATPAK_ID").is_err() {
        cache_dir = cache_dir.join(APP_DIR_NAME)
    }

    cache_dir
}

pub fn get_data_dir() -> io::Result<PathBuf> {
    let mut data_dir = if let Some(path) = env::var_os("XDG_DATA_HOME") {
        PathBuf::from(path)
    }
    else if let Some(home_path) = env::var_os("HOME") {
        PathBuf::from(home_path).join(".local").join("share")
    }
    else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Cannot find data directory. $XDG_DATA_HOME or $HOME is not set.",
        ));
    };

    // If not running inside flatpak, we need to add a separate subdir for the app.
    if env::var("FLATPAK_ID").is_err() {
        data_dir = data_dir.join(APP_DIR_NAME)
    }

    Ok(data_dir)
}

/// Create a local file. Local path is based on remote host name and remote file path.
/// Will overwrite any existing files.
pub fn create_file(host: &Host, remote_file_path: &str, mut metadata: FileMetadata, contents: Vec<u8>) -> io::Result<String> {
    let (dir_path, file_path) = convert_to_local_paths(host, remote_file_path);

    fs::create_dir_all(&dir_path)?;

    metadata.local_path = Some(file_path.clone());
    let metadata_file_path = convert_to_local_metadata_path(host, remote_file_path);
    let metadata_file = fs::OpenOptions::new().write(true).create(true).open(metadata_file_path)?;

    fs::write(&file_path, contents)?;
    serde_yaml::to_writer(metadata_file, &metadata).map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

    Ok(file_path)
}

/// Updates existing local file. File has to exist and have accompanying metadata file.
pub fn write_file(local_file_path: &String, contents: Vec<u8>) -> io::Result<()> {
    // Verify, just in case, that path belongs to cache directory.
    let cache_dir = get_cache_dir();
    if Path::new(local_file_path).ancestors().all(|ancestor| ancestor != cache_dir.as_path()) {
        Err(io::Error::new(io::ErrorKind::Other, "Path does not belong to cache directory"))
    }
    else {
        fs::write(local_file_path, contents)?;
        Ok(())
    }
}

pub fn write_file_metadata(metadata: FileMetadata) -> io::Result<()> {
    let local_file_path = metadata.local_path.clone()
        .ok_or(io::Error::new(io::ErrorKind::Other, "Metadata does not contain local file path"))?;

    let metadata_path = get_metadata_path(&local_file_path);
    let metadata_file = fs::OpenOptions::new().write(true).create(true).open(metadata_path)?;

    serde_yaml::to_writer(metadata_file, &metadata)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

    Ok(())
}

/// Removes local copy of the (possible) content file and metadata file.
pub fn remove_file(path: &String) -> io::Result<()> {
    // Verify, just in case, that path belongs to cache directory.
    let cache_dir = get_cache_dir();
    if Path::new(path).ancestors().all(|ancestor| ancestor != cache_dir.as_path()) {
        return Err(io::Error::new(io::ErrorKind::Other, "Path does not belong to cache directory"));
    }

    if path.ends_with(METADATA_SUFFIX) {
        // Some files may not be written to disk even though metadata might still exist.
        if let Some(file_path) = get_content_file_path(path) {
            fs::remove_file(file_path)?;
        }

        fs::remove_file(path)?;
    }
    else {
        fs::remove_file(get_metadata_path(path))?;
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn read_file(local_file_path: &str) -> io::Result<(FileMetadata, Vec<u8>)> {
    let contents = fs::read(local_file_path)?;

    let metadata_path = get_metadata_path(local_file_path);
    let metadata_string = fs::read_to_string(metadata_path)?;
    let metadata: FileMetadata = serde_yaml::from_str(&metadata_string).map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

    Ok((metadata, contents))
}

pub fn read_file_metadata(local_file_path: &String) -> io::Result<FileMetadata> {
    let metadata_path = get_metadata_path(local_file_path);
    let metadata_string = fs::read_to_string(metadata_path)?;
    let metadata: FileMetadata = serde_yaml::from_str(&metadata_string).map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

    Ok(metadata)
}

/// Provides the local metadata file path based on remote host name and remote file path.
pub fn convert_to_local_metadata_path(host: &Host, remote_file_path: &str) -> String {
    let (_, file_path) = convert_to_local_paths(host, remote_file_path);
    get_metadata_path(&file_path)
}

pub fn get_content_file_path(metadata_path: &str) -> Option<String> {
    if let Some(file_path) = metadata_path.strip_suffix(METADATA_SUFFIX) {
        if Path::new(&file_path).is_file() {
            return Some(file_path.to_string());
        }
    }

    None
}

pub fn get_metadata_path(local_file_path: &str) -> String {
    format!("{}{}", local_file_path, METADATA_SUFFIX)
}

/// Provides the local directory and file paths based on remote host name and remote file path.
pub fn convert_to_local_paths(host: &Host, remote_file_path: &str) -> (String, String) {
    let cache_dir = file_handler::get_cache_dir();
    let file_dir = cache_dir.join(host.name.clone());

    // Using only hash as the file name would suffice but providing some parts of
    // the file path and name will help the user to identify the file in e.g. text editor.
    let hash = sha256::hash(remote_file_path.as_bytes());
    let mut components = Path::new(remote_file_path).components().rev();

    let mut file_name = hash;

    for _ in 0..MAX_PATH_COMPONENTS {
        if let Some(next_component) = components.next() {
            file_name = format!("{}_{}", next_component.as_os_str().to_string_lossy(), file_name);
        }
        else {
            break;
        }
    }

    (
        Path::new(&file_dir).to_string_lossy().to_string(),
        Path::new(&file_dir).join(file_name).to_string_lossy().to_string(),
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileMetadata {
    /// When download was completed and file saved.
    pub download_time: DateTime<Utc>,
    pub local_path: Option<String>,
    pub remote_path: String,
    pub remote_file_hash: String,
    pub owner_uid: u32,
    pub owner_gid: u32,
    pub permissions: u32,
    /// Temporary files will be deleted when they're no longer used (usually after uploading).
    pub temporary: bool,
}

impl FileMetadata {
    pub fn update_hash(&mut self, new_hash: String) {
        self.remote_file_hash = new_hash;
    }
}

use std::{
    path::Path,
    path::PathBuf,
    fs,
    io,
    env,
};
use sha256;
use serde_derive::{ Serialize, Deserialize };
use serde_yaml;
use chrono::{DateTime, Utc};
use crate::Host;
use crate::file_handler;


const MAX_PATH_COMPONENTS: u8 = 2;
const APP_DIR_NAME: &str = "lightkeeper";


pub fn get_config_dir() -> io::Result<PathBuf> {
    let config_dir;
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        config_dir = PathBuf::from(path);
    }
    else if let Some(path) = env::var_os("HOME") {
        config_dir = PathBuf::from(path).join(".config");
    }
    else {
        return Err(io::Error::new(io::ErrorKind::Other, "Cannot find configuration directory. $XDG_CONFIG_HOME or $HOME is not set."));
    }

    // If running inside flatpak, there's no need to add a separate subdir for the app.
    if env::var("FLATPAK_ID").is_ok() {
        Ok(config_dir)
    }
    else {
        Ok(config_dir.join(APP_DIR_NAME))
    }
}

pub fn get_cache_dir() -> io::Result<PathBuf> {
    let mut cache_dir;
    if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        cache_dir = PathBuf::from(path);
    }
    else if let Some(home_path) = env::var_os("HOME") {
        cache_dir = PathBuf::from(home_path).join(".cache");
    }
    else {
        return Err(io::Error::new(io::ErrorKind::Other, "Cannot find cache directory. $XDG_CACHE_HOME or $HOME is not set."));
    }

    // If not running inside flatpak, we need to add a separate subdir for the app.
    if env::var("FLATPAK_ID").is_err() {
        cache_dir = cache_dir.join(APP_DIR_NAME)
    }

    Ok(cache_dir)
}

/// Create a local file. Local path is based on remote host name and remote file path.
/// Will overwrite any existing files.
pub fn create_file(host: &Host, remote_file_path: &String, file_mode: i32, file_contents: Vec<u8>) -> io::Result<String> {
    let file_dir = host.name.clone();
    if !Path::new(&file_dir).is_dir() {
        fs::create_dir(&file_dir)?;
    }

    let file_path = convert_to_local_path(host, remote_file_path);
    let metadata_file_path = convert_to_local_metadata_path(host, remote_file_path);
    let metadata_file = fs::OpenOptions::new().write(true).create(true).open(metadata_file_path)?;
    let metadata = FileMetadata {
        download_time: Utc::now(),
        remote_path: remote_file_path.clone(),
        remote_file_hash: sha256::digest(file_contents.as_slice()),
        mode: file_mode,
        temporary: true,
    };

    fs::write(&file_path, file_contents)?;
    serde_yaml::to_writer(metadata_file, &metadata)
               .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

    Ok(file_path)
}

/// Removes local copy of the file.
pub fn remove_file(local_file_path: &String) -> io::Result<()> {
    // TODO: path validation and limits just in case?
    fs::remove_file(local_file_path)?;
    Ok(())
}

pub fn read_file(local_file_path: &String) -> io::Result<(FileMetadata, Vec<u8>)> {
    // TODO: path validation and limits just in case?
    let contents = fs::read(&local_file_path)?;

    let metadata_path = get_metadata_path(local_file_path);
    let metadata_string = fs::read_to_string(metadata_path)?;
    let metadata: FileMetadata = serde_yaml::from_str(&metadata_string)
                                            .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

    Ok((metadata, contents))
}

/// Provides the local metadata file path based on remote host name and remote file path.
pub fn convert_to_local_metadata_path(host: &Host, remote_file_path: &String) -> String {
    let file_path = convert_to_local_path(host, remote_file_path);
    get_metadata_path(&file_path)
}

pub fn get_metadata_path(local_file_path: &String) -> String {
    format!("{}.metadata.yml", local_file_path)
}

/// Provides the local file path based on remote host name and remote file path.
pub fn convert_to_local_path(host: &Host, remote_file_path: &String) -> String {
    let cache_dir = file_handler::get_cache_dir().unwrap();
    let file_dir = cache_dir.join(host.name.clone());

    // Using only hash as the file name would suffice but providing some parts of
    // the file path and name will help the user to identify the file in e.g. text editor.
    let hash = sha256::digest(remote_file_path.clone());
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

    Path::new(&file_dir).join(file_name).to_string_lossy().to_string()
}


#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileMetadata {
    /// When download was completed and file saved.
    pub download_time: DateTime<Utc>,
    pub remote_path: String,
    pub remote_file_hash: String,
    pub mode: i32,
    /// Temporary files will be deleted when they're no longer used.
    pub temporary: bool,
}
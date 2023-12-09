use std::{
    path::Path,
    path::PathBuf,
    fs,
    io,
    env,
};
use serde_derive::{ Serialize, Deserialize };
use chrono::{DateTime, Utc};
use crate::Host;
use crate::file_handler;


const MAX_PATH_COMPONENTS: u8 = 2;
const APP_DIR_NAME: &str = "lightkeeper";
const METADATA_SUFFIX : &str = ".metadata.yml";


pub fn get_config_dir() -> io::Result<PathBuf> {
    let config_dir;
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        config_dir = PathBuf::from(path);
    }
    else if let Some(path) = env::var_os("HOME") {
        config_dir = PathBuf::from(path).join(".config");
    }
    else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Cannot find configuration directory. $XDG_CONFIG_HOME or $HOME is not set."
        ));
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
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Cannot find cache directory. $XDG_CACHE_HOME or $HOME is not set."
        ));
    }

    // If not running inside flatpak, we need to add a separate subdir for the app.
    if env::var("FLATPAK_ID").is_err() {
        cache_dir = cache_dir.join(APP_DIR_NAME)
    }

    Ok(cache_dir)
}

/// Create a local file. Local path is based on remote host name and remote file path.
/// Will overwrite any existing files.
pub fn create_file(host: &Host, remote_file_path: &String, mut metadata: FileMetadata, contents: Vec<u8>) -> io::Result<String> {
    let (dir_path, file_path) = convert_to_local_paths(host, remote_file_path);

    if !Path::new(&dir_path).is_dir() {
        fs::create_dir_all(&dir_path)?;
    }

    metadata.local_path = Some(file_path.clone());
    let metadata_file_path = convert_to_local_metadata_path(host, remote_file_path);
    let metadata_file = fs::OpenOptions::new().write(true).create(true).open(metadata_file_path)?;


    fs::write(&file_path, contents)?;
    serde_yaml::to_writer(metadata_file, &metadata)
               .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

    Ok(file_path)
}

pub fn list_cached_files(only_metadata_files: bool) -> io::Result<Vec<String>> {
    let cache_dir = file_handler::get_cache_dir()?;
    let mut files = Vec::new();


    // Nice drifting...
    for subdirectory in fs::read_dir(cache_dir)? {
        match subdirectory {
            Ok(subdirectory) => {
                if subdirectory.path().is_dir() && subdirectory.file_name() != "qmlcachedir" {

                    for entry in fs::read_dir(subdirectory.path())? {
                        match entry {
                            Ok(entry) => {
                                if entry.path().is_file() {
                                    let file_path = entry.path().to_string_lossy().to_string();

                                    if file_path.ends_with(METADATA_SUFFIX) {
                                        if !only_metadata_files {
                                            if let Some(content_file_path) = get_content_file_path(&file_path) {
                                                files.push(content_file_path.to_string());
                                            }
                                        }

                                        files.push(file_path);
                                    }
                                }
                            }
                            Err(error) => {
                                log::error!("Error while reading cache directory: {}", error);
                            }
                        }
                    }

                }
            }
            Err(error) => {
                log::error!("Error while reading cache directory: {}", error);
            }
        }
    }

    Ok(files)
}

/// Updates existing local file. File has to exist and have accompanying metadata file.
pub fn write_file(local_file_path: &String, contents: Vec<u8>) -> io::Result<()> {
    fs::write(local_file_path, contents)?;
    Ok(())
}

/// Removes local copy of the (possible) content file and metadata file.
pub fn remove_file(path: &String) -> io::Result<()> {
    // Verify, just in case, that path belongs to cache directory.
    let cache_dir = get_cache_dir().unwrap();
    if Path::new(path).ancestors().all(|ancestor| ancestor != cache_dir.as_path()) {
        panic!()
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

pub fn read_file(local_file_path: &String) -> io::Result<(FileMetadata, Vec<u8>)> {
    // Verify, just in case, that path belongs to cache directory.
    let cache_dir = get_cache_dir().unwrap();
    if Path::new(local_file_path).ancestors().all(|ancestor| ancestor != cache_dir.as_path()) {
        panic!()
    }

    let contents = fs::read(local_file_path)?;

    let metadata_path = get_metadata_path(local_file_path);
    let metadata_string = fs::read_to_string(metadata_path)?;
    let metadata: FileMetadata = serde_yaml::from_str(&metadata_string)
                                            .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

    Ok((metadata, contents))
}

/// Provides the local metadata file path based on remote host name and remote file path.
pub fn convert_to_local_metadata_path(host: &Host, remote_file_path: &String) -> String {
    let (_, file_path) = convert_to_local_paths(host, remote_file_path);
    get_metadata_path(&file_path)
}

pub fn get_content_file_path(metadata_path: &String) -> Option<String> {
    let file_path = metadata_path.strip_suffix(METADATA_SUFFIX).unwrap().to_string();
    if Path::new(&file_path).is_file() {
        Some(file_path)
    }
    else {
        None
    }
}

pub fn get_metadata_path(local_file_path: &String) -> String {
    format!("{}{}", local_file_path, METADATA_SUFFIX)
}

/// Provides the local directory and file paths based on remote host name and remote file path.
pub fn convert_to_local_paths(host: &Host, remote_file_path: &String) -> (String, String) {
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

    (
        Path::new(&file_dir).to_string_lossy().to_string(),
        Path::new(&file_dir).join(file_name).to_string_lossy().to_string()
    )
}


#[derive(Serialize, Deserialize)]
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
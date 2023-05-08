use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::fs;

use strum_macros::EnumString;

use crate::file_handler;

const CACHE_FILE_NAME: &str = "cache.yml";

// Simple cache implementation.
// cached-package was tested but it required a lot of dependencies.
// The needs are currently simple, so this is the simpler approach.
pub struct Cache<K, V> {
    time_to_live: Duration,
    data: HashMap<K, V>,
    last_access: HashMap<K, Instant>,
}

impl <K: Eq + std::hash::Hash + Clone, V: Clone> Cache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            time_to_live: ttl,
            data: HashMap::new(),
            last_access: HashMap::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        let value = self.data.get(key);

        if let Some(value) = value {
            if self.last_access.get(key).unwrap().elapsed() > self.time_to_live {
                self.data.remove(key);
                self.last_access.remove(key);
                None
            }
            else {
                self.last_access.insert(key.clone(), Instant::now());
                Some(value.clone())
            }
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.data.insert(key.clone(), value);
        self.last_access.insert(key, Instant::now());
    }

    /// Writes cache contents to disk in YAML format so they can be loaded on application start.
    pub fn write_to_disk(&self) -> Result<(), String> where K: serde::Serialize, V: serde::Serialize {
        log::debug!("Writing cache to disk.");

        let config_dir = file_handler::get_config_dir().map_err(|error| format!("Cannot find cache directory: {}", error))?;
        fs::create_dir_all(&config_dir).map_err(|error| format!("Error while creating configuration directory: {}", error))?;
        let file_path = config_dir.join(CACHE_FILE_NAME);
        let serialized = serde_yaml::to_string(&self.data).unwrap();
        fs::write(file_path, serialized).map_err(|error| format!("Error while writing cache to disk: {}", error))?;
        Ok(())
    }

    /// Read the cache from disk.
    pub fn read_from_disk(&mut self) -> Result<(), String> where K: serde::de::DeserializeOwned, V: serde::de::DeserializeOwned {
        log::debug!("Reading cache from disk.");

        let config_dir = file_handler::get_config_dir().map_err(|error| format!("Cannot find cache directory: {}", error))?;
        let file_path = config_dir.join(CACHE_FILE_NAME);

        let serialized = fs::read_to_string(file_path).map_err(|error| format!("Error while reading cache from disk: {}", error))?;
        self.data = serde_yaml::from_str(&serialized).map_err(|error| format!("Cache is in invalid format: {}", error))?;
        Ok(())
    }
}

/// Describes if cache is global or host-specific.
#[derive(Clone, EnumString)]
pub enum CacheScope {
    Global,
    Host,
}
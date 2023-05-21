use std::collections::HashMap;
use std::time::{SystemTime, Duration, Instant};
use std::fs;

use serde_derive::{Serialize, Deserialize};
use strum_macros::EnumString;

use crate::file_handler;

const CACHE_FILE_NAME: &str = "cache.yml";


// Simple cache implementation.
// cached-crate was tested but it required a lot of dependencies.
// The needs are currently simple, so this is subjectively a better approach.
pub struct Cache<K, V> {
    time_to_live: Duration,
    final_time_to_live: Duration,
    data: HashMap<K, CacheEntry<V>>,
}

impl <K: Eq + std::hash::Hash + Clone, V: Clone> Cache<K, V> {
    pub fn new(time_to_live: u64, final_time_to_live: u64) -> Self {
        Self {
            time_to_live: Duration::from_secs(time_to_live),
            final_time_to_live: Duration::from_secs(final_time_to_live),
            data: HashMap::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        let entry = self.data.get(key)?.clone();

        if entry.last_access.elapsed() > self.final_time_to_live {
            self.data.remove(key);
            None
        }
        else if entry.last_access.elapsed() > self.time_to_live {
            None
        }
        else {
            // Update last access time.
            self.data.insert(key.clone(), CacheEntry {
                value: entry.value.clone(),
                last_access: Instant::now(),
            });
            Some(entry.value)
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.data.insert(key.clone(), CacheEntry {
            value: value.clone(),
            last_access: Instant::now(),
        });
    }

    /// Writes cache contents to disk in YAML format so they can be loaded on application start.
    pub fn write_to_disk(&self) -> Result<usize, String> where K: serde::Serialize, V: serde::Serialize {
        // TODO: don't write sensitive information. Modules should define if they can contain sensitive info.
        let config_dir = file_handler::get_config_dir().map_err(|error| format!("Cannot find cache directory: {}", error))?;
        fs::create_dir_all(&config_dir).map_err(|error| format!("Error while creating configuration directory: {}", error))?;
        let file_path = config_dir.join(CACHE_FILE_NAME);
        let serialized = serde_yaml::to_string(&self.data).unwrap();
        fs::write(file_path, serialized).map_err(|error| format!("Error while writing cache to disk: {}", error))?;
        Ok(self.data.len() as usize)
    }

    /// Read the cache from disk.
    pub fn read_from_disk(&mut self) -> Result<usize, String> where K: serde::de::DeserializeOwned, V: serde::de::DeserializeOwned {
        let config_dir = file_handler::get_config_dir().map_err(|error| format!("Cannot find cache directory: {}", error))?;
        let file_path = config_dir.join(CACHE_FILE_NAME);

        let serialized = fs::read_to_string(file_path).map_err(|error| format!("Error while reading cache from disk: {}", error))?;
        self.data = serde_yaml::from_str(&serialized).map_err(|error| format!("Cache is in invalid format: {}", error))?;

        // Go through the cache to clean up any old entries.
        let keys = self.data.keys().cloned().collect::<Vec<K>>();
        for key in keys {
            let _ = self.get(&key);
        }

        Ok(self.data.len() as usize)
    }
}

/// Describes if cache is global or host-specific.
#[derive(Clone, EnumString)]
pub enum CacheScope {
    Global,
    Host,
}

#[derive(Serialize, Deserialize, Clone)]
struct CacheEntry<V> {
    value: V,
    // Instant is a better choice here than SystemTime.
    #[serde(serialize_with = "serialize_instant", deserialize_with = "deserialize_instant")]
    last_access: Instant,
}

fn serialize_instant<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer
{
    // Stored as UNIX timestamp (in seconds).
    let secs_since_epoch = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() - instant.elapsed().as_secs();
    serializer.serialize_u64(secs_since_epoch)
}

fn deserialize_instant<'de, D>(deserializer: D) -> Result<Instant, D::Error> where D: serde::Deserializer<'de>
{
    let secs_since_epoch = serde::Deserialize::deserialize(deserializer)?;
    let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(secs_since_epoch);
    let instant = Instant::now() - SystemTime::now().duration_since(system_time).unwrap();
    Ok(instant)
}

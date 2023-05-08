use strum_macros::EnumString;
use std::collections::HashMap;
use std::time::{Duration, Instant};

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
}

/// Describes if cache is global or host-specific.
#[derive(Clone, EnumString)]
pub enum CacheScope {
    Global,
    Host,
}
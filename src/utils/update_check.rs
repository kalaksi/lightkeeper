/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::fs;
use std::io::Read;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::file_handler;

const VERSION_URL: &str = "https://github.com/kalaksi/lightkeeper/releases/latest/download/version.json";
const CACHE_FILE: &str = "update-check.yml";
const MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Serialize, Deserialize)]
struct Cache {
    checked_at: u64,
    latest_version: String,
}

#[derive(Deserialize)]
struct VersionFile {
    version: String,
}

/// Returns `(cached_version, cache_is_fresh)`.
pub fn load_cache() -> (Option<String>, bool) {
    let path = file_handler::get_cache_dir().join(CACHE_FILE);
    let Ok(contents) = fs::read_to_string(&path) else {
        return (None, false);
    };
    let Ok(cache) = serde_yaml::from_str::<Cache>(&contents) else {
        return (None, false);
    };
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let fresh = now.saturating_sub(cache.checked_at) < MAX_AGE.as_secs();
    let version = if cache.latest_version.is_empty() {
        None
    } else {
        Some(cache.latest_version)
    };
    (version, fresh)
}

/// Fetches `version.json` from the latest GitHub release and writes the cache. Returns the version on success.
pub fn fetch_and_store() -> Option<String> {
    let previous = load_cache().0.unwrap_or_default();
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(3)))
        .build()
        .new_agent();

    let mut response = match agent
        .get(VERSION_URL)
        .header("User-Agent", concat!("Lightkeeper/", env!("CARGO_PKG_VERSION")))
        .call()
    {
        Ok(response) => response,
        Err(error) => {
            log::debug!("Update check request failed: {}", error);
            write_cache(&previous);
            return None;
        }
    };

    if response.status().as_u16() != 200 {
        log::debug!("Update check HTTP status: {}", response.status());
        write_cache(&previous);
        return None;
    }

    let mut body = String::new();
    if let Err(error) = response.body_mut().as_reader().read_to_string(&mut body) {
        log::debug!("Update check body read failed: {}", error);
        write_cache(&previous);
        return None;
    }

    let version = match serde_json::from_str::<VersionFile>(&body) {
        Ok(file) if !file.version.is_empty() => file.version.trim().to_string(),
        Ok(_) => {
            log::debug!("Update check version.json has empty version");
            write_cache(&previous);
            return None;
        }
        Err(error) => {
            log::debug!("Update check version.json parse failed: {}", error);
            write_cache(&previous);
            return None;
        }
    };

    write_cache(&version);
    Some(version)
}

fn write_cache(latest_version: &str) {
    let checked_at = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let cache = Cache {
        checked_at,
        latest_version: latest_version.to_string(),
    };
    let cache_dir = file_handler::get_cache_dir();
    if let Err(error) = fs::create_dir_all(&cache_dir) {
        log::debug!("Update check cache dir create failed: {}", error);
        return;
    }
    let path = cache_dir.join(CACHE_FILE);
    if let Err(error) = fs::write(&path, serde_yaml::to_string(&cache).unwrap_or_default()) {
        log::debug!("Update check cache write failed: {}", error);
    }
}

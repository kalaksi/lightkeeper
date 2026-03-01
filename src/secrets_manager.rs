/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use keyring::Entry;
use keyring::Error as KeyringError;

use crate::error::LkError;

const SERVICE_NAME: &str = "lightkeeper";
pub const KEYRING_PLACEHOLDER_PREFIX: &str = "keyring:";

pub fn get(key: &str) -> Result<Option<String>, LkError> {
    let entry = Entry::new(SERVICE_NAME, key)?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(KeyringError::NoEntry) => {
            log::warn!("Secret not found in keyring: {}", key);
            Ok(None)
        },
        Err(e) => Err(e.into()),
    }
}

pub fn set(key: &str, value: &str) -> Result<(), LkError> {
    let entry = Entry::new(SERVICE_NAME, key)?;
    entry.set_password(value)?;
    let attrs = HashMap::from([("application", "lightkeeper")]);
    entry.update_attributes(&attrs)?;
    Ok(())
}

pub fn delete(key: &str) -> Result<(), LkError> {
    let entry = Entry::new(SERVICE_NAME, key)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(KeyringError::NoEntry) => {
            log::warn!("Secret not found in keyring: {}", key);
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}

/// Stores secret settings in the keyring and returns a new map with placeholders for secret values.
/// source_id is "group:<id>" or "host:<id>". Call before writing config.
pub fn store_connector_secrets(
    connector_id: &str,
    source_id: &str,
    settings: &HashMap<String, String>,
    secret_keys: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for (key, value) in settings {
        if !secret_keys.contains_key(key) {
            result.insert(key.clone(), value.clone());
            continue;
        }
        if value.starts_with(KEYRING_PLACEHOLDER_PREFIX) {
            result.insert(key.clone(), value.clone());
            continue;
        }
        let lookup_key = secret_lookup_key(connector_id, source_id, key);
        if value.is_empty() {
            let _ = delete(&lookup_key);
        }
        else {
            if let Err(e) = set(&lookup_key, value) {
                log::warn!("Failed to store secret for {} {}: {}", connector_id, key, e);
            }
            result.insert(key.clone(), format!("{}{}", KEYRING_PLACEHOLDER_PREFIX, source_id));
        }
    }
    result
}

/// Keyring key for placeholder "keyring:SOURCE_ID". Works for any connector and setting.
/// source_id is "group:<id>" or "host:<id>".
pub fn secret_lookup_key(connector_id: &str, source_id: &str, setting_key: &str) -> String {
    format!("{}:{}:{}", connector_id, source_id, setting_key)
}

impl From<KeyringError> for LkError {
    fn from(e: KeyringError) -> Self {
        LkError::other(e.to_string())
    }
}

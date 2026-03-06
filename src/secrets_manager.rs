/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use keyring::Entry;
use keyring::Error as KeyringError;

use crate::error::LkError;

const SERVICE_NAME: &str = "lightkeeper";
pub const KEYRING_PREFIX: &str = "keyring:";

pub struct SecretsManager {
    cache: HashMap<String, Option<String>>,
}

impl SecretsManager {
    pub fn new() -> Self {
        SecretsManager {
            cache: HashMap::new(),
        }
    }

    pub fn get(&mut self, key: &str) -> Result<Option<String>, LkError> {
        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let value = crate::secrets_manager::get(key)?;
        self.cache.insert(key.to_string(), value.clone());
        Ok(value)
    }
}

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
    let label = format!("lightkeeper/{}", key);
    let attrs = HashMap::from([("application", "lightkeeper"), ("label", label.as_str())]);
    entry.update_attributes(&attrs)?;
    Ok(())
}

pub fn delete(key: &str) -> Result<(), LkError> {
    let entry = Entry::new(SERVICE_NAME, key)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(error) => {
            log::warn!("Failed to remove keyring secret for {}: {}", key, error);
            Err(error.into())
        }
    }
}

/// Keyring key for placeholder "keyring:SOURCE_ID". Works for any module and setting.
/// source_id is "group:<id>" or "host:<id>".
pub fn secret_lookup_key(connector_id: &str, source_id: &str, setting_key: &str) -> String {
    format!("{}:{}:{}", connector_id, source_id, setting_key)
}

impl From<KeyringError> for LkError {
    fn from(e: KeyringError) -> Self {
        LkError::other(e.to_string())
    }
}

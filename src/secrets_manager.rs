/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

#[cfg(feature = "native")]
use keyring::Entry;
#[cfg(feature = "native")]
use keyring::Error as KeyringError;

#[cfg(feature = "flatpak")]
use std::sync::OnceLock;

use crate::error::LkError;
use crate::utils::strip_unprintable;

const SERVICE_NAME: &str = "lightkeeper";
pub const NATIVE_KEYRING_PREFIX: &str = "keyring:";
pub const PORTAL_KEYRING_PREFIX: &str = "pkeyring:";

#[cfg(feature = "native")]
pub const KEYRING_PREFIX: &str = NATIVE_KEYRING_PREFIX;
#[cfg(feature = "flatpak")]
pub const KEYRING_PREFIX: &str = PORTAL_KEYRING_PREFIX;

#[cfg(feature = "native")]
pub const INACTIVE_KEYRING_PREFIX: &str = PORTAL_KEYRING_PREFIX;
#[cfg(feature = "flatpak")]
pub const INACTIVE_KEYRING_PREFIX: &str = NATIVE_KEYRING_PREFIX;

#[cfg(feature = "flatpak")]
static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
#[cfg(feature = "flatpak")]
static OO7_KEYRING: OnceLock<oo7::Keyring> = OnceLock::new();

#[cfg(feature = "flatpak")]
fn runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Failed to create tokio runtime")
    })
}

#[cfg(feature = "flatpak")]
fn keyring() -> &'static oo7::Keyring {
    OO7_KEYRING.get_or_init(|| {
        runtime().block_on(async {
            oo7::Keyring::new().await.expect("Failed to initialize oo7 keyring")
        })
    })
}

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

#[cfg(feature = "native")]
pub fn get(key: &str) -> Result<Option<String>, LkError> {
    let entry = Entry::new(SERVICE_NAME, key)?;
    match entry.get_password() {
        Ok(value) => Ok(Some(strip_unprintable(&value))),
        Err(KeyringError::NoEntry) => {
            log::warn!("Secret not found in keyring: {}", key);
            Ok(None)
        },
        Err(e) => Err(e.into()),
    }
}

#[cfg(feature = "flatpak")]
pub fn get(key: &str) -> Result<Option<String>, LkError> {
    let attributes = [("service", SERVICE_NAME), ("key", key)];
    let items = runtime().block_on(keyring().search_items(&attributes))?;
    match items.first() {
        Some(item) => {
            let secret = runtime().block_on(item.secret())?;
            let value = match &secret {
                oo7::Secret::Text(text) => text.clone(),
                oo7::Secret::Blob(bytes) => String::from_utf8_lossy(bytes).to_string(),
            };
            Ok(Some(strip_unprintable(&value)))
        }
        None => {
            log::warn!("Secret not found in keyring: {}", key);
            Ok(None)
        }
    }
}

#[cfg(feature = "native")]
pub fn set(key: &str, value: &str) -> Result<(), LkError> {
    let value = strip_unprintable(value);
    let entry = Entry::new(SERVICE_NAME, key)?;
    entry.set_password(&value)?;
    let label = format!("lightkeeper/{}", key);
    let attrs = HashMap::from([("application", "lightkeeper"), ("label", label.as_str())]);
    entry.update_attributes(&attrs)?;
    Ok(())
}

#[cfg(feature = "flatpak")]
pub fn set(key: &str, value: &str) -> Result<(), LkError> {
    let value = strip_unprintable(value);
    let label = format!("lightkeeper/{}", key);
    let attributes = [("service", SERVICE_NAME), ("key", key)];
    runtime().block_on(keyring().create_item(&label, &attributes, oo7::Secret::text(&value), true))?;
    Ok(())
}

#[cfg(feature = "native")]
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

#[cfg(feature = "flatpak")]
pub fn delete(key: &str) -> Result<(), LkError> {
    let attributes = [("service", SERVICE_NAME), ("key", key)];
    runtime().block_on(keyring().delete(&attributes))?;
    Ok(())
}

/// Keyring key for placeholder "keyring:SOURCE_ID". Works for any module and setting.
/// source_id is "group:<id>" or "host:<id>".
pub fn secret_lookup_key(connector_id: &str, source_id: &str, setting_key: &str) -> String {
    format!("{}:{}:{}", connector_id, source_id, setting_key)
}

#[cfg(feature = "native")]
impl From<KeyringError> for LkError {
    fn from(e: KeyringError) -> Self {
        LkError::other(e.to_string())
    }
}

#[cfg(feature = "flatpak")]
impl From<oo7::Error> for LkError {
    fn from(e: oo7::Error) -> Self {
        LkError::other(e.to_string())
    }
}

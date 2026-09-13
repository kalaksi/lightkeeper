/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::sync::Mutex;

use lightkeeper::error::LkError;
use lightkeeper::secrets_manager::SecretStore;
use lightkeeper::utils::strip_unprintable;

#[derive(Default)]
pub struct MemorySecretStore {
    entries: Mutex<HashMap<String, String>>,
}

impl MemorySecretStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStore for MemorySecretStore {
    fn get(&self, key: &str) -> Result<Option<String>, LkError> {
        Ok(self.entries.lock().unwrap().get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) -> Result<(), LkError> {
        let value = strip_unprintable(value);
        self.entries.lock().unwrap().insert(key.to_string(), value);
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), LkError> {
        self.entries.lock().unwrap().remove(key);
        Ok(())
    }
}

/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::sync::Arc;

use super::api::ConfigBackend;
use crate::configuration::{self, Configuration};
use crate::error::LkError;
use crate::secrets_manager::{KEYRING_PREFIX, SecretStore, secret_lookup_key};

pub struct LocalConfigBackend {
    config_dir: String,
    secret_store: Arc<dyn SecretStore>,
}

impl LocalConfigBackend {
    pub fn new(config_dir: String, secret_store: Arc<dyn SecretStore>) -> Self {
        LocalConfigBackend {
            config_dir,
            secret_store,
        }
    }
}

impl ConfigBackend for LocalConfigBackend {
    fn get_config(&self) -> Result<(Configuration, configuration::Hosts, configuration::Groups), LkError> {
        Ok(Configuration::read(&self.config_dir)?)
    }

    fn update_config(
        &self,
        main_config: Configuration,
        hosts: configuration::Hosts,
        groups: configuration::Groups,
    ) -> Result<(), LkError> {
        Configuration::write_all_configs_transactional(&self.config_dir, &main_config, &hosts, &groups)?;
        if let Err(error) = Configuration::clear_config_backups(&self.config_dir) {
            log::warn!("Failed to clear configuration backups: {}", error);
        }
        Ok(())
    }

    fn get_secret(&self, source_id: &str, module_id: &str, setting_key: &str) -> Result<Option<String>, LkError> {
        let lookup_key = secret_lookup_key(module_id, source_id, setting_key);
        self.secret_store.get(&lookup_key)
    }

    fn store_secret(
        &self,
        source_id: &str,
        module_id: &str,
        setting_key: &str,
        secret_value: &str,
    ) -> Result<String, LkError> {
        let lookup_key = secret_lookup_key(module_id, source_id, setting_key);
        self.secret_store.set(&lookup_key, secret_value)?;
        Ok(format!("{}{}", KEYRING_PREFIX, lookup_key))
    }

    fn remove_secret(&self, source_id: &str, module_id: &str, setting_key: &str) -> Result<(), LkError> {
        let lookup_key = secret_lookup_key(module_id, source_id, setting_key);
        self.secret_store.delete(&lookup_key)
    }
}

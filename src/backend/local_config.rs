/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use super::api::ConfigBackend;
use crate::configuration::{self, Configuration};
use crate::error::LkError;

pub struct LocalConfigBackend {
    config_dir: String,
}

impl LocalConfigBackend {
    pub fn new(config_dir: String) -> Self {
        LocalConfigBackend { config_dir }
    }
}

impl ConfigBackend for LocalConfigBackend {
    fn get_config(&self) -> Result<(Configuration, configuration::Hosts, configuration::Groups), LkError> {
        Ok(Configuration::read(&self.config_dir)?)
    }

    fn update_config(&self, main_config: Configuration, hosts: configuration::Hosts, groups: configuration::Groups) -> Result<(), LkError> {
        Configuration::write_main_config(&self.config_dir, &main_config)?;
        Configuration::write_hosts_config(&self.config_dir, &hosts)?;
        Configuration::write_groups_config(&self.config_dir, &groups)?;
        Ok(())
    }
}

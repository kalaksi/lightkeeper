/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::path::PathBuf;
use std::sync::mpsc;

use crate::Configuration;
use crate::configuration;
use crate::error::LkError;
use crate::file_handler;
use crate::frontend;
use crate::CoreComponents;
use crate::ModuleFactory;
use std::sync::Arc;

pub struct CoreRuntime {
    pub core: CoreComponents,
    pub config_dir: String,
}

impl CoreRuntime {
    pub fn new(
        main_config: &Configuration,
        hosts_config: &configuration::Hosts,
        config_dir: String,
    ) -> Result<Self, LkError> {
        Self::new_with_module_factory(
            main_config,
            hosts_config,
            Arc::new(ModuleFactory::new()),
            config_dir,
        )
    }

    pub fn new_with_module_factory(
        main_config: &Configuration,
        hosts_config: &configuration::Hosts,
        module_factory: Arc<ModuleFactory>,
        config_dir: String,
    ) -> Result<Self, LkError> {

        let mut core = crate::initialize_core(main_config, hosts_config, module_factory)?;

        if main_config.preferences.refresh_hosts_on_start {
            let host_ids = core.monitor_manager.refresh_platform_info_all();
            log::info!("Initialized {} host(s)", host_ids.len());
        }

        Ok(CoreRuntime { core, config_dir })
    }

    pub fn default_socket_path() -> Result<PathBuf, LkError> {
        Ok(file_handler::get_data_dir()?.join("core.sock"))
    }

    pub fn new_update_receiver(&mut self) -> mpsc::Receiver<frontend::UIUpdate> {
        let (sender, receiver) = mpsc::channel();
        self.core.host_manager.borrow_mut().add_observer(sender);
        receiver
    }

    pub fn stop(&mut self) {
        self.core.command_handler.stop();
        self.core.monitor_manager.stop();
        self.core.host_manager.borrow_mut().stop();
        self.core.connection_manager.stop();
    }
}

impl Drop for CoreRuntime {
    fn drop(&mut self) {
        self.stop();
    }
}

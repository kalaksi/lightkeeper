/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::path::PathBuf;
use std::sync::mpsc;

use crate::command_handler::CommandButtonData;
use crate::Configuration;
use crate::configuration;
use crate::configuration::CustomCommandConfig;
use crate::error::LkError;
use crate::file_handler;
use crate::frontend;
use crate::frontend::DisplayData;
use crate::CoreComponents;
use crate::ModuleFactory;
use std::collections::HashMap;
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
        let mut core =
            crate::initialize_core_with_module_factory(main_config, hosts_config, module_factory)?;

        if main_config.preferences.refresh_hosts_on_start {
            let host_ids = core.monitor_manager.refresh_platform_info_all();
            log::info!("Initialized {} host(s)", host_ids.len());
        }

        Ok(CoreRuntime { core, config_dir })
    }

    pub fn default_socket_path() -> Result<PathBuf, LkError> {
        Ok(file_handler::get_data_dir()?.join("core.sock"))
    }

    pub fn display_data(&self) -> DisplayData {
        self.core.host_manager.borrow().get_display_data()
    }

    pub fn new_update_receiver(&mut self) -> mpsc::Receiver<frontend::UIUpdate> {
        let (sender, receiver) = mpsc::channel();
        self.core.host_manager.borrow_mut().add_observer(sender);
        receiver
    }

    pub fn execute_command(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> Result<u64, LkError> {
        self.core.command_handler.execute(host_id, command_id, parameters)
    }

    pub fn commands_for_host(&self, host_id: &str) -> HashMap<String, CommandButtonData> {
        self.core.command_handler.get_commands_for_host(host_id.to_string())
    }

    pub fn command_for_host(&self, host_id: &str, command_id: &str) -> Option<CommandButtonData> {
        self.core.command_handler
            .get_command_for_host(&host_id.to_string(), &command_id.to_string())
    }

    pub fn custom_commands_for_host(&self, host_id: &str) -> HashMap<String, CustomCommandConfig> {
        self.core.command_handler.get_custom_commands_for_host(&host_id.to_string())
    }

    pub fn all_host_categories(&self, host_id: &str) -> Vec<String> {
        self.core.monitor_manager.get_all_host_categories(host_id)
    }

    pub fn verify_host_key(&self, host_id: &str, connector_id: &str, key_id: &str) {
        self.core.command_handler.verify_host_key(
            &host_id.to_string(),
            &connector_id.to_string(),
            &key_id.to_string(),
        );
    }

    pub fn interrupt_invocation(&self, invocation_id: u64) {
        self.core.command_handler.interrupt_invocation(invocation_id);
    }

    pub fn refresh_host_monitors(&mut self, host_id: &str) {
        for category in self.core.monitor_manager.get_all_host_categories(host_id) {
            let _ = self.core.monitor_manager.refresh_monitors_of_category(host_id, &category);
        }
    }

    pub fn refresh_platform_info(&mut self, host_id: &str) {
        self.core.monitor_manager.refresh_platform_info(host_id);
    }

    pub fn refresh_platform_info_all(&mut self) -> Vec<String> {
        self.core.monitor_manager.refresh_platform_info_all()
    }

    pub fn refresh_monitors_for_command(&mut self, host_id: &str, command_id: &str) -> Vec<u64> {
        let command = match self.command_for_host(host_id, command_id) {
            Some(command) => command,
            None => return Vec::new(),
        };

        if command.display_options.parent_id.is_empty() {
            self.core.monitor_manager.refresh_monitors_of_category(host_id, &command.display_options.category)
        }
        else {
            self.core.monitor_manager.refresh_monitors_by_id(
                &host_id.to_string(),
                &command.display_options.parent_id,
            )
        }
    }

    pub fn refresh_monitors_of_category(&mut self, host_id: &str, category: &str) -> Vec<u64> {
        self.core.monitor_manager.refresh_monitors_of_category(host_id, category)
    }

    pub fn refresh_certificate_monitors(&mut self) -> Vec<u64> {
        self.core.monitor_manager.refresh_certificate_monitors()
    }

    pub fn resolve_text_editor_path(&self, host_id: &str, command_id: &str, parameters: &[String]) -> Option<String> {
        if let Some(path) = parameters.first().cloned() {
            return Some(path);
        }

        self.core.command_handler.get_connector_message(&host_id.to_string(), &command_id.to_string())
    }

    pub fn download_editable_file(
        &mut self,
        host_id: &str,
        command_id: &str,
        remote_file_path: &str,
    ) -> Result<u64, LkError> {
        let (invocation_id, _) = self.core.command_handler.download_editable_file(
            &host_id.to_string(),
            &command_id.to_string(),
            &remote_file_path.to_string(),
        )?;
        Ok(invocation_id)
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

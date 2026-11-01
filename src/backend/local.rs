/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::api::{CommandBackend, LocalBackendApi};
use crate::command_handler::CommandButtonData;
use crate::configuration;
use crate::error::LkError;
use crate::file_handler;
use crate::CoreComponents;

pub struct LocalCommandBackend {
    core: Rc<RefCell<CoreComponents>>,
}

impl LocalCommandBackend {
    pub fn new(core: Rc<RefCell<CoreComponents>>) -> Self {
        LocalCommandBackend { core }
    }

    fn connector_message(&self, host_id: &str, command_id: &str) -> Option<String> {
        self.core
            .borrow()
            .command_handler
            .get_connector_message(&host_id.to_string(), &command_id.to_string())
    }
}

impl CommandBackend for LocalCommandBackend {
    fn stop(&mut self) {
        let mut core = self.core.borrow_mut();
        core.command_handler.stop();
        core.monitor_manager.stop();
    }

    fn refresh_host_monitors(&mut self, host_id: &str) {
        let mut core = self.core.borrow_mut();
        for category in core.monitor_manager.get_all_host_categories(host_id) {
            let _invocation_ids = core.monitor_manager.refresh_monitors_of_category(host_id, &category);
        }
    }

    fn commands_for_host(&self, host_id: &str) -> Result<HashMap<String, CommandButtonData>, LkError> {
        Ok(self
            .core
            .borrow()
            .command_handler
            .get_commands_for_host(host_id.to_string()))
    }

    fn command_for_host(&self, host_id: &str, command_id: &str) -> Result<Option<CommandButtonData>, LkError> {
        Ok(self
            .core
            .borrow()
            .command_handler
            .get_command_for_host(&host_id.to_string(), &command_id.to_string()))
    }

    fn custom_commands_for_host(&self, host_id: &str) -> Result<HashMap<String, configuration::CustomCommandConfig>, LkError> {
        Ok(self
            .core
            .borrow()
            .command_handler
            .get_custom_commands_for_host(&host_id.to_string()))
    }

    fn all_host_categories(&self, host_id: &str) -> Result<Vec<String>, LkError> {
        Ok(self.core.borrow().monitor_manager.get_all_host_categories(host_id))
    }

    fn execute_command(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> Result<u64, LkError> {
        self.core.borrow_mut().command_handler.execute(host_id, command_id, parameters)
    }

    fn interrupt_invocation(&self, invocation_id: u64) {
        self.core.borrow().command_handler.interrupt_invocation(invocation_id);
    }

    fn verify_host_key(&self, host_id: &str, connector_id: &str, key_id: &str) {
        self.core.borrow().command_handler.verify_host_key(
            &host_id.to_string(),
            &connector_id.to_string(),
            &key_id.to_string(),
        );
    }

    fn initialize_host(&mut self, host_id: &str) {
        self.core.borrow_mut().monitor_manager.refresh_platform_info(host_id);
    }

    fn initialize_hosts(&mut self) -> Result<Vec<String>, LkError> {
        Ok(self.core.borrow_mut().monitor_manager.refresh_platform_info_all())
    }

    fn refresh_monitors_for_command(&mut self, host_id: &str, command_id: &str) -> Result<Vec<u64>, LkError> {
        let command = match self.command_for_host(host_id, command_id)? {
            Some(command) => command,
            None => return Ok(Vec::new()),
        };

        let mut core = self.core.borrow_mut();
        if command.display_options.parent_id.is_empty() {
            Ok(core
                .monitor_manager
                .refresh_monitors_of_category(host_id, &command.display_options.category))
        }
        else {
            Ok(core
                .monitor_manager
                .refresh_monitors_by_id(&host_id.to_string(), &command.display_options.parent_id))
        }
    }

    fn refresh_monitors_of_category(&mut self, host_id: &str, category: &str) -> Result<Vec<u64>, LkError> {
        Ok(self
            .core
            .borrow_mut()
            .monitor_manager
            .refresh_monitors_of_category(host_id, category))
    }

    fn refresh_certificate_monitors(&mut self) -> Result<Vec<u64>, LkError> {
        Ok(self.core.borrow_mut().monitor_manager.refresh_certificate_monitors())
    }

    fn resolve_text_editor_path(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> Result<Option<String>, LkError> {
        if let Some(path) = parameters.first().cloned() {
            Ok(Some(path))
        }
        else {
            Ok(self.connector_message(host_id, command_id))
        }
    }

    fn download_editable_file(&mut self, host_id: &str, command_id: &str, remote_file_path: &str) -> Result<(u64, String), LkError> {
        self.core.borrow_mut().command_handler.download_editable_file(
            &host_id.to_string(),
            &command_id.to_string(),
            &remote_file_path.to_string(),
        )
    }

    fn upload_file(&mut self, host_id: &str, command_id: &str, local_file_path: &str, force: bool) -> Result<u64, LkError> {
        self.core.borrow_mut().command_handler.upload_file(
            &host_id.to_string(),
            &command_id.to_string(),
            &local_file_path.to_string(),
            force,
        )
    }

    fn upload_file_from_cache(
        &mut self,
        host_id: &str,
        command_id: &str,
        remote_file_path: &str,
        force: bool,
    ) -> Result<u64, LkError> {
        let mut core = self.core.borrow_mut();
        let path = core.command_handler.cache_file_path_for_remote(host_id, remote_file_path)?;
        core.command_handler
            .upload_file(&host_id.to_string(), &command_id.to_string(), &path, force)
    }

    fn write_cached_file(&mut self, host_id: &str, remote_file_path: &str, new_contents: Vec<u8>) -> Result<(), LkError> {
        let local_path = self
            .core
            .borrow()
            .command_handler
            .cache_file_path_for_remote(host_id, remote_file_path)?;
        file_handler::write_file(&local_path, new_contents)?;
        Ok(())
    }

    fn remove_cached_file(&mut self, host_id: &str, remote_file_path: &str) -> Result<(), LkError> {
        let local_path = self
            .core
            .borrow()
            .command_handler
            .cache_file_path_for_remote(host_id, remote_file_path)?;
        file_handler::remove_file(&local_path)?;
        Ok(())
    }

    fn has_cached_file_changed(&self, host_id: &str, remote_file_path: &str, new_contents: &[u8]) -> Result<bool, LkError> {
        let content_hash = crate::utils::sha256::hash(new_contents);
        self.core
            .borrow()
            .command_handler
            .has_file_changed(host_id, remote_file_path, &content_hash)
    }

    fn local_backend(&self) -> Option<&dyn LocalBackendApi> {
        Some(self)
    }
}

impl LocalBackendApi for LocalCommandBackend {
    fn remote_terminal_command(&self, host_id: &str, command_id: &str, parameters: &[String]) -> crate::utils::ShellCommand {
        self.core.borrow().command_handler.open_remote_terminal_command(
            &host_id.to_string(),
            &command_id.to_string(),
            parameters,
        )
    }

    fn open_external_terminal(&self, host_id: &str, command_id: &str, parameters: Vec<String>) {
        self.core.borrow().command_handler.open_external_terminal(
            &host_id.to_string(),
            &command_id.to_string(),
            parameters,
        );
    }

    fn remote_text_editor_command(&self, host_id: &str, remote_file_path: &str) -> crate::utils::ShellCommand {
        self.core
            .borrow()
            .command_handler
            .open_remote_text_editor(&host_id.to_string(), remote_file_path)
    }

    fn open_external_text_editor(&self, host_id: &str, command_id: &str, local_file_path: &str) -> Result<(), LkError> {
        self.core.borrow().command_handler.open_external_text_editor(
            &host_id.to_string(),
            &command_id.to_string(),
            &local_file_path.to_string(),
        );
        Ok(())
    }
}

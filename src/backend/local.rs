/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::sync::mpsc;

use crate::command_handler::{CommandButtonData, CommandHandler};
use crate::configuration;
use crate::connection_manager::ConnectorRequest;
use crate::frontend;
use crate::host_manager::StateUpdateMessage;
use crate::monitor_manager::MonitorManager;

use super::api::{CommandBackend, LocalBackendApi};

#[derive(Default)]
pub struct LocalCommandBackend {
    command_handler: CommandHandler,
    monitor_manager: MonitorManager,
}

impl LocalCommandBackend {
    pub fn new(command_handler: CommandHandler, monitor_manager: MonitorManager) -> Self {
        LocalCommandBackend {
            command_handler,
            monitor_manager,
        }
    }

    fn connector_message(&self, host_id: &str, command_id: &str) -> Option<String> {
        self.command_handler
            .get_connector_message(&host_id.to_string(), &command_id.to_string())
    }
}

impl CommandBackend for LocalCommandBackend {
    fn configure(
        &mut self,
        hosts_config: &configuration::Hosts,
        preferences: &configuration::Preferences,
        request_sender: mpsc::Sender<ConnectorRequest>,
        update_sender: mpsc::Sender<StateUpdateMessage>,
        _frontend_update_sender: mpsc::Sender<frontend::UIUpdate>,
    ) {
        self.monitor_manager.configure(hosts_config, request_sender.clone(), update_sender.clone());
        self.command_handler.configure(hosts_config, preferences, request_sender, update_sender);
    }

    fn start_processing_responses(&mut self) {
        self.monitor_manager.start_processing_responses();
        self.command_handler.start_processing_responses();
    }

    fn stop(&mut self) {
        self.command_handler.stop();
        self.monitor_manager.stop();
    }

    fn refresh_host_monitors(&mut self, host_id: &str) {
        for category in self.monitor_manager.get_all_host_categories(host_id) {
            let _invocation_ids = self.monitor_manager.refresh_monitors_of_category(host_id, &category);
        }
    }

    fn commands_for_host(&self, host_id: &str) -> HashMap<String, CommandButtonData> {
        self.command_handler.get_commands_for_host(host_id.to_string())
    }

    fn command_for_host(&self, host_id: &str, command_id: &str) -> Option<CommandButtonData> {
        self.command_handler.get_command_for_host(&host_id.to_string(), &command_id.to_string())
    }

    fn custom_commands_for_host(&self, host_id: &str) -> HashMap<String, configuration::CustomCommandConfig> {
        self.command_handler.get_custom_commands_for_host(&host_id.to_string())
    }

    fn all_host_categories(&self, host_id: &str) -> Vec<String> {
        self.monitor_manager.get_all_host_categories(host_id)
    }

    fn execute_command(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> u64 {
        self.command_handler.execute(host_id, command_id, parameters)
    }

    fn interrupt_invocation(&self, invocation_id: u64) {
        self.command_handler.interrupt_invocation(invocation_id);
    }

    fn verify_host_key(&self, host_id: &str, connector_id: &str, key_id: &str) {
        self.command_handler.verify_host_key(&host_id.to_string(), &connector_id.to_string(), &key_id.to_string());
    }

    fn initialize_host(&mut self, host_id: &str) {
        self.monitor_manager.refresh_platform_info(host_id);
    }

    fn initialize_hosts(&mut self) -> Vec<String> {
        self.monitor_manager.refresh_platform_info_all()
    }

    fn refresh_monitors_for_command(&mut self, host_id: &str, command_id: &str) -> Vec<u64> {
        let command = match self.command_for_host(host_id, command_id) {
            Some(command) => command,
            None => return Vec::new(),
        };

        if command.display_options.parent_id.is_empty() {
            self.monitor_manager.refresh_monitors_of_category(host_id, &command.display_options.category)
        } else {
            self.monitor_manager.refresh_monitors_by_id(&host_id.to_string(), &command.display_options.parent_id)
        }
    }

    fn refresh_monitors_of_category(&mut self, host_id: &str, category: &str) -> Vec<u64> {
        self.monitor_manager.refresh_monitors_of_category(host_id, category)
    }

    fn refresh_certificate_monitors(&mut self) -> Vec<u64> {
        self.monitor_manager.refresh_certificate_monitors()
    }

    fn resolve_text_editor_path(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> Option<String> {
        if let Some(path) = parameters.first().cloned() {
            Some(path)
        } else {
            self.connector_message(host_id, command_id)
        }
    }

    fn download_editable_file(&mut self, host_id: &str, command_id: &str, remote_file_path: &str) -> (u64, String) {
        self.command_handler.download_editable_file(
            &host_id.to_string(),
            &command_id.to_string(),
            &remote_file_path.to_string(),
        )
    }

    fn upload_file(&mut self, host_id: &str, command_id: &str, local_file_path: &str) -> u64 {
        self.command_handler.upload_file(
            &host_id.to_string(),
            &command_id.to_string(),
            &local_file_path.to_string(),
        )
    }

    fn upload_file_from_editor(&mut self, host_id: &str, command_id: &str, remote_file_path: &str, contents: Vec<u8>) -> u64 {
        self.command_handler.upload_file_from_editor_contents(
            &host_id.to_string(),
            &command_id.to_string(),
            &remote_file_path.to_string(),
            contents,
        )
    }

    fn write_file(&mut self, local_file_path: &str, new_contents: Vec<u8>) {
        self.command_handler.write_file(&local_file_path.to_string(), new_contents);
    }

    fn remove_file(&mut self, local_file_path: &str) {
        self.command_handler.remove_file(&local_file_path.to_string());
    }

    fn has_file_changed(&self, local_file_path: &str, new_contents: &[u8]) -> bool {
        self.command_handler.has_file_changed(&local_file_path.to_string(), new_contents)
    }

    fn local_backend(&self) -> Option<&dyn LocalBackendApi> {
        Some(self)
    }
}

impl LocalBackendApi for LocalCommandBackend {
    fn remote_terminal_command(
        &self,
        host_id: &str,
        command_id: &str,
        parameters: &[String],
    ) -> crate::utils::ShellCommand {
        self.command_handler.open_remote_terminal_command(
            &host_id.to_string(),
            &command_id.to_string(),
            parameters,
        )
    }

    fn open_external_terminal(&self, host_id: &str, command_id: &str, parameters: Vec<String>) {
        self.command_handler.open_external_terminal(&host_id.to_string(), &command_id.to_string(), parameters);
    }

    fn remote_text_editor_command(&self, host_id: &str, remote_file_path: &str,) -> crate::utils::ShellCommand {
        self.command_handler.open_remote_text_editor(&host_id.to_string(), remote_file_path)
    }

    fn open_external_text_editor(&self, host_id: &str, command_id: &str, remote_file_path: &str,) -> String {
        self.command_handler.open_external_text_editor(
            &host_id.to_string(),
            &command_id.to_string(),
            &remote_file_path.to_string(),
        )
    }
}

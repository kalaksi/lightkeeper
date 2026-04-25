/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::sync::mpsc;

use crate::command_handler::CommandButtonData;
use crate::configuration;
use crate::connection_manager::ConnectorRequest;
use crate::error::LkError;
use crate::frontend;
use crate::host_manager::StateUpdateMessage;

//
// Traits for command backend API and local backend API.
//

pub trait CommandBackend {
    fn configure(
        &mut self,
        hosts_config: &configuration::Hosts,
        preferences: &configuration::Preferences,
        request_sender: mpsc::Sender<ConnectorRequest>,
        update_sender: mpsc::Sender<StateUpdateMessage>,
        frontend_update_sender: mpsc::Sender<frontend::UIUpdate>,
    );
    fn start_processing_responses(&mut self);
    fn stop(&mut self);
    fn refresh_host_monitors(&mut self, host_id: &str);
    fn commands_for_host(&self, host_id: &str) -> Result<HashMap<String, CommandButtonData>, LkError>;
    fn command_for_host(&self, host_id: &str, command_id: &str) -> Result<Option<CommandButtonData>, LkError>;
    fn custom_commands_for_host(&self, host_id: &str) -> Result<HashMap<String, configuration::CustomCommandConfig>, LkError>;
    fn all_host_categories(&self, host_id: &str) -> Result<Vec<String>, LkError>;
    fn execute_command(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> Result<u64, LkError>;
    fn interrupt_invocation(&self, invocation_id: u64);
    fn verify_host_key(&self, host_id: &str, connector_id: &str, key_id: &str);
    fn initialize_host(&mut self, host_id: &str);
    fn initialize_hosts(&mut self) -> Result<Vec<String>, LkError>;
    fn refresh_monitors_for_command(&mut self, host_id: &str, command_id: &str) -> Result<Vec<u64>, LkError>;
    fn refresh_monitors_of_category(&mut self, host_id: &str, category: &str) -> Result<Vec<u64>, LkError>;
    fn refresh_certificate_monitors(&mut self) -> Result<Vec<u64>, LkError>;
    fn resolve_text_editor_path(
        &mut self,
        host_id: &str,
        command_id: &str,
        parameters: &[String],
    ) -> Result<Option<String>, LkError>;
    fn download_editable_file(
        &mut self,
        host_id: &str,
        command_id: &str,
        remote_file_path: &str,
    ) -> Result<(u64, String), LkError>;
    fn upload_file(&mut self, host_id: &str, command_id: &str, local_file_path: &str) -> Result<u64, LkError>;
    fn upload_file_from_cache(&mut self, host_id: &str, command_id: &str, remote_file_path: &str) -> Result<u64, LkError>;
    fn write_cached_file(&mut self, host_id: &str, remote_file_path: &str, new_contents: Vec<u8>) -> Result<(), LkError>;
    fn remove_cached_file(&mut self, host_id: &str, remote_file_path: &str) -> Result<(), LkError>;
    fn has_cached_file_changed(&self, host_id: &str, remote_file_path: &str, new_contents: &[u8]) -> Result<bool, LkError>;

    fn local_backend(&self) -> Option<&dyn LocalBackendApi> {
        None
    }
}

pub trait ConfigBackend {
    fn get_config(&self) -> Result<(configuration::Configuration, configuration::Hosts, configuration::Groups), LkError>;

    fn update_config(
        &self,
        main_config: configuration::Configuration,
        hosts: configuration::Hosts,
        groups: configuration::Groups,
    ) -> Result<(), LkError>;
}

pub trait LocalBackendApi {
    fn remote_terminal_command(&self, host_id: &str, command_id: &str, parameters: &[String]) -> crate::utils::ShellCommand;
    fn open_external_terminal(&self, host_id: &str, command_id: &str, parameters: Vec<String>);
    fn remote_text_editor_command(&self, host_id: &str, remote_file_path: &str) -> crate::utils::ShellCommand;
    fn open_external_text_editor(&self, host_id: &str, command_id: &str, remote_file_path: &str) -> String;
}

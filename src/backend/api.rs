/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::sync::mpsc;

use crate::command_handler::CommandButtonData;
use crate::configuration;
use crate::connection_manager::ConnectorRequest;
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
    fn commands_for_host(&self, host_id: &str) -> HashMap<String, CommandButtonData>;
    fn command_for_host(&self, host_id: &str, command_id: &str) -> Option<CommandButtonData>;
    fn custom_commands_for_host(&self, host_id: &str) -> HashMap<String, configuration::CustomCommandConfig>;
    fn all_host_categories(&self, host_id: &str) -> Vec<String>;
    fn execute_command(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> u64;
    fn interrupt_invocation(&self, invocation_id: u64);
    fn verify_host_key(&self, host_id: &str, connector_id: &str, key_id: &str);
    fn initialize_host(&mut self, host_id: &str);
    fn initialize_hosts(&mut self) -> Vec<String>;
    fn refresh_monitors_for_command(&mut self, host_id: &str, command_id: &str) -> Vec<u64>;
    fn refresh_monitors_of_category(&mut self, host_id: &str, category: &str) -> Vec<u64>;
    fn refresh_certificate_monitors(&mut self) -> Vec<u64>;
    fn resolve_text_editor_path(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> Option<String>;
    fn download_editable_file(&mut self, host_id: &str, command_id: &str, remote_file_path: &str) -> (u64, String);
    fn upload_file(&mut self, host_id: &str, command_id: &str, local_file_path: &str) -> u64;
    fn upload_file_from_editor(&mut self, host_id: &str, command_id: &str, remote_file_path: &str, contents: Vec<u8>) -> u64;
    fn write_file(&mut self, local_file_path: &str, new_contents: Vec<u8>);
    fn remove_file(&mut self, local_file_path: &str);
    fn has_file_changed(&self, local_file_path: &str, new_contents: &[u8]) -> bool;
    fn local_backend(&self) -> Option<&dyn LocalBackendApi> {
        None
    }
}
pub trait LocalBackendApi {
    fn remote_terminal_command(
        &self,
        host_id: &str,
        command_id: &str,
        parameters: &[String],
    ) -> crate::utils::ShellCommand;
    fn open_external_terminal(&self, host_id: &str, command_id: &str, parameters: Vec<String>);
    fn remote_text_editor_command(
        &self,
        host_id: &str,
        remote_file_path: &str,
    ) -> crate::utils::ShellCommand;
    fn open_external_text_editor(
        &self,
        host_id: &str,
        command_id: &str,
        remote_file_path: &str,
    ) -> String;
}

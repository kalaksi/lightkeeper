
use core::panic;
use std::sync::Arc;
use std::sync::mpsc;
use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;
use serde_derive::{Serialize, Deserialize};
use std::cell::RefCell;
use std::rc::Rc;

use crate::configuration::Hosts;
use crate::error::*;
use crate::file_handler;
use crate::file_handler::write_file_metadata;
use crate::host_manager::HostManager;
use crate::module::command::UIAction;
use crate::module::connection::request_response::RequestResponse;
use crate::module::module_factory::ModuleFactory;
use crate::module::ModuleType;
use crate::utils::*;
use crate::{
    configuration::Preferences,
    Host,
    host_manager::StateUpdateMessage,
    frontend::DisplayOptions,
    connection_manager::*, 
};

use crate::module::{
    command::Command,
    command::CommandResult,
};

// Default needs to be implemented because of Qt QObject requirements.
#[derive(Default)]
pub struct CommandHandler {
    /// Host name is the first key, command id is the second key.
    commands: Arc<Mutex<HashMap<String, HashMap<String, Command>>>>,
    /// For communication to ConnectionManager.
    request_sender: Option<mpsc::Sender<ConnectorRequest>>,
    /// Channel to send state updates to HostManager.
    state_update_sender: Option<mpsc::Sender<StateUpdateMessage>>,
    /// Preferences from config file.
    preferences: Preferences,
    /// Effective host configurations.
    hosts_config: Hosts,
    /// Every execution gets an invocation ID. Valid ID numbers begin from 1.
    invocation_id_counter: u64,

    // Shared resources.
    /// Mainly for getting up-to-date Host-datas.
    host_manager: Rc<RefCell<HostManager>>,
    module_factory: Arc<ModuleFactory>,

    response_sender_prototype: Option<mpsc::Sender<RequestResponse>>,
    response_receiver: Option<mpsc::Receiver<RequestResponse>>,
    response_receiver_thread: Option<thread::JoinHandle<()>>,
}

impl CommandHandler {
    pub fn new(host_manager: Rc<RefCell<HostManager>>, module_factory: Arc<ModuleFactory>) -> Self {
        CommandHandler {
            host_manager: host_manager.clone(),
            module_factory: module_factory,
            ..Default::default()
        }
    }

    pub fn configure(&mut self,
                     hosts_config: &Hosts,
                     preferences: &Preferences,
                     request_sender: mpsc::Sender<ConnectorRequest>,
                     state_update_sender: mpsc::Sender<StateUpdateMessage>) {

        self.commands.lock().unwrap().clear();

        self.request_sender = Some(request_sender);
        self.state_update_sender = Some(state_update_sender);

        self.preferences = preferences.clone();
        self.hosts_config = hosts_config.clone();

        for (host_id, host_config) in hosts_config.hosts.iter() {
            for (command_id, command_config) in host_config.effective.commands.iter() {

                let command_spec = crate::module::ModuleSpecification::new(command_id, &command_config.version);
                if let Some(command) = self.module_factory.new_command(&command_spec, &command_config.settings) {
                    self.add_command(host_id, command);
                }
            }
        }

        let (sender, receiver) = mpsc::channel::<RequestResponse>();
        self.response_sender_prototype = Some(sender);
        self.response_receiver = Some(receiver);
    }

    pub fn stop(&mut self) {
        self.new_response_sender()
            .send(RequestResponse::stop())
            .unwrap_or_else(|error| log::error!("Couldn't send exit token to command handler: {}", error));

        if let Some(thread) = self.response_receiver_thread.take() {
            thread.join().unwrap();
        }
    }

    pub fn new_response_sender(&self) -> mpsc::Sender<RequestResponse> {
        self.response_sender_prototype.clone().unwrap()
    }

    fn add_command(&mut self, host_id: &String, command: Command) {
        let mut commands = self.commands.lock().unwrap();
        commands.entry(host_id.clone()).or_insert(HashMap::new());

        let command_collection = commands.get_mut(host_id).unwrap();
        let module_spec = command.get_module_spec();

        // Only add if missing.
        command_collection.entry(module_spec.id).or_insert(command);
    }

    /// Returns invocation ID or 0 on error.
    pub fn execute(&mut self, host_id: &String, command_id: &String, parameters: &[String]) -> u64 {

        let host = self.host_manager.borrow().get_host(host_id);

        if !host.platform.is_set() {
            log::warn!("[{}] Executing command \"{}\" despite missing platform info", host_id, command_id);
        }

        let commands = self.commands.lock().unwrap();
        let command = &commands[host_id][command_id];

        let state_update_sender = self.state_update_sender.as_ref().unwrap().clone();

        let messages = match get_command_connector_messages(&host, command, parameters) {
            Ok(messages) => messages,
            Err(error) => {
                log::error!("Command failed: {}", error);
                state_update_sender.send(StateUpdateMessage {
                    host_name: host.name,
                    display_options: command.get_display_options(),
                    module_spec: command.get_module_spec(),
                    command_result: Some(CommandResult::new_error(error)),
                    ..Default::default()
                }).unwrap();
                return 0;
            }
        };

        self.invocation_id_counter += 1;

        // Notify host state manager about new command, so it can keep track of pending invocations.
        state_update_sender.send(StateUpdateMessage {
            host_name: host.name.clone(),
            display_options: command.get_display_options(),
            module_spec: command.get_module_spec(),
            command_result: Some(CommandResult::pending()),
            ..Default::default()
        }).unwrap();

        let request_type = match command.get_display_options().action == UIAction::FollowOutput {
            true => RequestType::CommandFollowOutput { commands: messages },
            false => RequestType::Command { commands: messages }
        };

        // Send request to ConnectionManager.
        self.request_sender.as_ref().unwrap().send(ConnectorRequest {
            connector_spec: command.get_connector_spec(),
            source_id: command.get_module_spec().id,
            host: host.clone(),
            invocation_id: self.invocation_id_counter,
            request_type: request_type,
            response_sender: self.new_response_sender(),
        }).unwrap();

        self.invocation_id_counter
    }

    //
    // INTEGRATED COMMANDS
    //

    pub fn download_editable_file(&mut self, host_id: &String, command_id: &String, remote_file_path: &String) -> (u64, String) {
        let host = self.host_manager.borrow().get_host(host_id);
        let commands = self.commands.lock().unwrap();
        let command = &commands[host_id][command_id];

        let connector_messages = get_command_connector_messages(&host, command, &[remote_file_path.clone()]).map_err(|error| {
            log::error!("Command failed: {}", error);
            return;
        }).unwrap();

        let (_, local_file_path) = file_handler::convert_to_local_paths(&host, remote_file_path);
        self.invocation_id_counter += 1;

        self.request_sender.as_ref().unwrap().send(ConnectorRequest {
            connector_spec: command.get_connector_spec(),
            source_id: command.get_module_spec().id,
            host: host.clone(),
            invocation_id: self.invocation_id_counter,
            response_sender: self.new_response_sender(),
            request_type: RequestType::Download {
                remote_file_path: connector_messages[0].to_owned(),
            },
        }).unwrap();

        (self.invocation_id_counter, local_file_path)
    }

    pub fn upload_file(&mut self, host_id: &String, command_id: &String, local_file_path: &String) -> u64 {
        let host = self.host_manager.borrow().get_host(host_id);
        let commands = self.commands.lock().unwrap();
        let command = &commands[host_id][command_id];

        let state_update_sender = self.state_update_sender.as_ref().unwrap().clone();
        self.invocation_id_counter += 1;

        match file_handler::read_file(local_file_path) {
            Ok((mut metadata, contents)) => {
                let local_file_hash = sha256::digest(contents.as_slice());

                if local_file_hash == metadata.remote_file_hash {
                    state_update_sender.send(StateUpdateMessage {
                        host_name: host.name,
                        display_options: command.get_display_options(),
                        module_spec: command.get_module_spec(),
                        command_result: Some(CommandResult::new_error("File is unchanged")),
                        ..Default::default()
                    }).unwrap();
                }
                else {
                    metadata.update_hash(local_file_hash);

                    self.request_sender.as_ref().unwrap().send(ConnectorRequest {
                        connector_spec: command.get_connector_spec(),
                        source_id: command.get_module_spec().id,
                        host: host.clone(),
                        invocation_id: self.invocation_id_counter,
                        response_sender: self.new_response_sender(),
                        request_type: RequestType::Upload {
                            local_file_path: local_file_path.clone(),
                            metadata: metadata,
                        },
                    }).unwrap();
                }
            },
            Err(error) => {
                log::error!("Error while reading file: {}", error);
                state_update_sender.send(StateUpdateMessage {
                    host_name: host.name,
                    display_options: command.get_display_options(),
                    module_spec: command.get_module_spec(),
                    command_result: Some(CommandResult::new_critical_error(error)),
                    ..Default::default()
                }).unwrap();
            }
        }

        self.invocation_id_counter
    }

    pub fn verify_host_key(&self, host_id: &String, connector_id: &String, key_id: &String) {
        let host = self.host_manager.borrow().get_host(host_id);
        // Version numbers aren't currently used, so it's hardcoded here.
        let module_spec = crate::module::ModuleSpecification::new_with_type(&connector_id, "0.0.1", ModuleType::Connector);

        self.request_sender.as_ref().unwrap().send(ConnectorRequest {
            connector_spec: Some(module_spec),
            source_id: String::new(),
            host: host.clone(),
            invocation_id: 0,
            response_sender: self.new_response_sender(),
            request_type: RequestType::KeyVerification {
                key_id: key_id.to_owned(),
            },
        }).unwrap();
    }

    pub fn open_remote_terminal_command(&self, host_id: &String, command_id: &String, parameters: &[String]) -> ShellCommand {
        let host = self.host_manager.borrow().get_host(host_id);
        let mut command = self.remote_ssh_command(&host);

        let commands = self.commands.lock().unwrap();
        let command_module = &commands[host_id][command_id];

        let connector_messages = get_command_connector_messages(&host, command_module, parameters).unwrap_or_else(|error| {
            log::error!("Command failed: {}", error);
            Vec::new()
        });

        command.arguments(connector_messages);
        ::log::debug!("Opening terminal with command: {}", command.to_string());
        command
    }

    // TODO: this will block the UI thread. Improve!
    pub fn open_external_terminal(&self, host_id: &String, command_id: &String, parameters: Vec<String>) {
        let command_args = self.open_remote_terminal_command(host_id, command_id, &parameters);

        log::debug!("Starting local process: {} {}", self.preferences.terminal, command_args.to_string());
        let result = ShellCommand::new()
            .arguments(vec![self.preferences.terminal.clone()])
            .arguments(self.preferences.terminal_args.clone())
            .arguments(command_args.to_vec())
            .execute();

        if let Err(error) = result {
            log::error!("Couldn't start terminal: {}", error);
        }
    }

    pub fn open_remote_text_editor(&self, host_id: &String, remote_file_path: &str) -> ShellCommand {
        let host = self.host_manager.borrow().get_host(host_id);
        let mut command = self.remote_ssh_command(&host);

        if self.preferences.sudo_remote_editor {
            command.argument("sudo");
        }

        command.argument(self.preferences.remote_text_editor.clone());
        command.argument(remote_file_path);
        command
    }

    // TODO: this will block the UI thread? Improve!
    /// Returns local file path where file was downloaded.
    pub fn open_external_text_editor(&self, host_id: &String, command_id: &String, remote_file_path: &String) -> String {
        let host = self.host_manager.borrow().get_host(host_id);
        let commands = self.commands.lock().unwrap();
        let command = &commands[host_id][command_id];

        let connector_messages = get_command_connector_messages(&host, command, &[remote_file_path.clone()]).map_err(|error| {
            log::error!("Command failed: {}", error);
            return;
        }).unwrap();

        self.request_sender.as_ref().unwrap().send(ConnectorRequest {
            connector_spec: command.get_connector_spec(),
            source_id: command.get_module_spec().id,
            host: host.clone(),
            // TODO: proper invocation id needed?
            invocation_id: 0,
            response_sender: self.new_response_sender(),
            request_type: RequestType::Download {
                remote_file_path: connector_messages[0].to_owned(),
            },
        }).unwrap();

        file_handler::convert_to_local_paths(&host, remote_file_path).1
    }


    //
    // RESPONSE HANDLING
    //

    pub fn start_processing_responses(&mut self) {
        let thread = Self::_start_processing_responses(
            self.commands.clone(),
            self.response_receiver.take().unwrap(),
            self.preferences.clone(),
            self.state_update_sender.as_ref().unwrap().clone()
        );

        self.response_receiver_thread = Some(thread);
    }

    fn _start_processing_responses(
        commands: Arc<Mutex<HashMap<String, HashMap<String, Command>>>>,
        receiver: mpsc::Receiver<RequestResponse>,
        preferences: Preferences,
        state_update_sender: mpsc::Sender<StateUpdateMessage>) -> thread::JoinHandle<()> {

        thread::spawn(move || {
            loop {
                let response = match receiver.recv() {
                    Ok(response) => response,
                    Err(error) => {
                        log::error!("Stopped receiver thread: {}", error);
                        return;
                    }
                };

                if response.stop {
                    log::debug!("Gracefully stopping receiver thread");
                    return;
                }

                let commands = commands.lock().unwrap();
                let command = &commands[&response.host.name][&response.source_id];
                let new_state_update_sender = state_update_sender.clone();

                match command.get_display_options().action {
                    UIAction::None |
                    UIAction::FollowOutput |
                    UIAction::DetailsDialog |
                    UIAction::TextView |
                    UIAction::TextDialog |
                    UIAction::LogView |
                    UIAction::LogViewWithTimeControls =>
                        Self::process_command_response(command, new_state_update_sender, response),
                    UIAction::TextEditor => {
                        match response.request_type.clone() {
                            RequestType::Download { .. } => {
                                if preferences.text_editor == crate::configuration::INTERNAL {
                                    Self::process_download_for_internal_editor(command, new_state_update_sender, response);
                                }
                                else {
                                    Self::process_download_for_external_editor(command, &preferences.text_editor, new_state_update_sender, response);
                                }
                            },
                            RequestType::Upload { local_file_path, metadata } => {
                                Self::process_upload_file_response(command, &local_file_path, metadata, false, new_state_update_sender, response);
                            },
                            _ => panic!("Invalid request type: {:?}", response.request_type)
                        }
                    },
                    _ => panic!("Unsupported UIAction"),
                }
            }
        })
    }

    fn process_command_response(
        command: &Command,
        state_update_sender: mpsc::Sender<StateUpdateMessage>,
        response: RequestResponse) {

        let command_id = &command.get_module_spec().id;
        let (messages, errors): (Vec<_>, Vec<_>) =  response.responses.into_iter().partition(Result::is_ok);
        let messages = messages.into_iter().map(Result::unwrap).collect::<Vec<_>>();
        let mut errors = errors.into_iter().map(Result::unwrap_err).collect::<Vec<_>>();

        let mut result;
        if !messages.is_empty() {
            result = command.process_responses(response.host.clone(), messages.clone());
            if let Err(error) = result {
                if error.kind == ErrorKind::NotImplemented {
                    let message = &messages[0];
                    // Wasn't implemented, try the other method.
                    result = command.process_response(response.host.clone(), message)
                                    .map_err(|error| LkError::from(error));
                }
                else {
                    result = Err(error);
                }
            }
        }
        else {
            result = Err(LkError::other("No responses received"));
        }

        let new_command_result = match result {
            Ok(mut command_result) => {
                let log_message = if command_result.message.len() > 5000 {
                    format!("{}...(long message cut)...", &command_result.message[..5000])
                }
                else {
                    command_result.message.clone()
                };

                log::debug!("[{}][{}] Command result received: {}", response.host.name, response.source_id, log_message);
                command_result.command_id = command.get_module_spec().id;
                Some(command_result)
            },
            Err(error) => {
                errors.push(error.set_source(command_id));
                None
            }
        };

        for error in errors.iter() {
            log::error!("[{}][{}] Error: {}", response.host.name, error.source_id, error.message);
        }

        state_update_sender.send(StateUpdateMessage {
            host_name: response.host.name,
            display_options: command.get_display_options(),
            module_spec: command.get_module_spec(),
            command_result: new_command_result,
            errors: errors,
            invocation_id: response.invocation_id,
            ..Default::default()
        }).unwrap();
    }


    fn process_download_for_internal_editor(command: &Command, state_update_sender: mpsc::Sender<StateUpdateMessage>, response: RequestResponse) {
        let message_result = &response.responses[0];

        let command_result = match message_result {
            Ok(response_message) => {
                let (_, contents) = file_handler::read_file(&response_message.message).unwrap();
                CommandResult::new_hidden(String::from_utf8(contents).unwrap())
            },
            Err(error) => {
                let error_message = format!("Error downloading file: {}", error);
                log::error!("{}", error_message);
                CommandResult::new_critical_error(error_message)
            }
        };

        state_update_sender.send(StateUpdateMessage {
            host_name: response.host.name,
            display_options: command.get_display_options(),
            module_spec: command.get_module_spec(),
            command_result: Some(command_result),
            invocation_id: response.invocation_id,
            ..Default::default()
        }).unwrap();
    }

    fn process_download_for_external_editor(
        command: &Command,
        text_editor: &String,
        state_update_sender: mpsc::Sender<StateUpdateMessage>,
        response: RequestResponse
    ) {
        let message_result = &response.responses[0];

        match message_result {
            Ok(response_message) => {
                let local_file = response_message.message.clone();
                log::debug!("Starting local process: {} {}", text_editor, local_file);

                // Blocks until finished.
                let result = ShellCommand::new()
                    .arguments(vec![text_editor, &local_file])
                    .execute();

                if let Err(error) = result {
                    log::error!("Couldn't start text editor: {}", error);
                    return;
                }
            },
            Err(error) => {
                let error_message = format!("Error downloading file: {}", error);
                log::error!("{}", error_message);

                state_update_sender.send(StateUpdateMessage {
                    host_name: response.host.name,
                    display_options: command.get_display_options(),
                    module_spec: command.get_module_spec(),
                    command_result: Some(CommandResult::new_critical_error(error_message)),
                    ..Default::default()
                }).unwrap();
            }
        };
    }

    fn process_upload_file_response(
        command: &Command,
        local_file_path: &String,
        new_metadata: file_handler::FileMetadata,
        remove_file: bool,
        state_update_sender: mpsc::Sender<StateUpdateMessage>,
        response: RequestResponse
    ) {
        let message_result = &response.responses[0];

        let command_result = match message_result {
            Ok(_) => {
                if remove_file {
                    file_handler::remove_file(local_file_path).unwrap();
                }
                else {
                    // Updates file hash in metadata.
                    write_file_metadata(new_metadata.clone()).unwrap();
                }

                CommandResult::new_info("File updated")
            },
            Err(error) => {
                let error_message = format!("Error uploading file: {}", error);
                log::error!("{}", error_message);
                CommandResult::new_critical_error(error_message)
            }
        };

        state_update_sender.send(StateUpdateMessage {
            host_name: response.host.name,
            display_options: command.get_display_options(),
            module_spec: command.get_module_spec(),
            command_result: Some(command_result),
            invocation_id: response.invocation_id,
            ..Default::default()
        }).unwrap();
    }


    //
    // HELPER FUNCTIONS
    //

    // Return value contains host's commands. `parameters` is not set since provided by data point later on.
    pub fn get_commands_for_host(&self, host_id: String) -> HashMap<String, CommandData> {
        if let Some(command_collection) = self.commands.lock().unwrap().get(&host_id) {
            command_collection.iter().map(|(command_id, command)| {
                (command_id.clone(), CommandData::new(command_id.clone(), command.get_display_options()))
            }).collect()
        }
        else {
            HashMap::new()
        }
    }

    pub fn get_command_for_host(&self, host_id: &String, command_id: &String) -> Option<CommandData> {
        let commands = self.commands.lock().unwrap();
        let command = commands.get(host_id).unwrap()
                              .get(command_id).unwrap();
        Some(CommandData::new(command_id.clone(), command.get_display_options()))
    }

    pub fn write_file(&mut self, local_file_path: &String, new_contents: Vec<u8>) {
        file_handler::write_file(local_file_path, new_contents).unwrap();
    }

    pub fn remove_file(&mut self, local_file_path: &String) {
        if file_handler::remove_file(local_file_path).is_ok() {
            log::debug!("Removed file {}", local_file_path);
        }
        else {
            log::error!("Failed to remove file {}", local_file_path);
        }
    }

    pub fn has_file_changed(&self, local_file_path: &String, new_contents: Vec<u8>) -> bool {
        match file_handler::read_file_metadata(local_file_path) {
            Ok(metadata) => {
                let content_hash = sha256::digest(new_contents.as_slice());
                content_hash != metadata.remote_file_hash
            },
            Err(error) => {
                log::error!("Error reading file metadata: {}", error);
                false
            }
        }
    }

    fn remote_ssh_command(&self, host: &Host) -> ShellCommand {
        let ssh_settings = self.hosts_config.hosts[&host.name].effective.connectors["ssh"].settings.clone();

        let remote_address = if !host.fqdn.is_empty() {
            host.fqdn.clone()
        }
        else {
            host.ip_address.to_string()
        };

        let mut command = ShellCommand::new();
        command.arguments(vec![
            String::from("ssh"),
            String::from("-t"),
            String::from("-p"), ssh_settings.get("port").unwrap_or(&String::from("22")).clone(),
        ]);

        if let Some(username) = ssh_settings.get("username") {
            command.arguments(vec![String::from("-l"), username.clone()]);
        }

        if let Some(private_key_path) = ssh_settings.get("private_key_path") {
            command.arguments(vec![String::from("-i"), private_key_path.clone()]);
        }

        command.argument(remote_address);
        command
    }
}

fn get_command_connector_messages(host: &Host, command: &Command, parameters: &[String]) -> Result<Vec<String>, LkError> {
    let mut all_messages: Vec<String> = Vec::new();

    match command.get_connector_messages(host.clone(), parameters.to_owned()) {
        Ok(messages) => all_messages.extend(messages),
        Err(error) => {
            if error.kind != ErrorKind::NotImplemented {
                return Err(LkError::from(error).set_source(command.get_module_spec().id))
            }
        }
    }

    match command.get_connector_message(host.clone(), parameters.to_owned()) {
        Ok(message) => all_messages.push(message),
        Err(error) => {
            if error.kind != ErrorKind::NotImplemented {
                return Err(LkError::from(error).set_source(command.get_module_spec().id))
            }
        }
    }

    all_messages.retain(|message| !message.is_empty());
    Ok(all_messages)
}


#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CommandData {
    pub command_id: String,
    pub command_params: Vec<String>,
    pub display_options: DisplayOptions,
}

impl CommandData {
    pub fn new(command_id: String, display_options: DisplayOptions) -> Self {
        CommandData {
            command_id: command_id,
            command_params: Vec::new(),
            display_options: display_options,
        }
    }
}
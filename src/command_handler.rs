
use std::sync::mpsc::Sender;
use std::collections::HashMap;
use serde_derive::{Serialize, Deserialize};
use std::process;
use std::cell::RefCell;
use std::rc::Rc;

use crate::host_manager::HostManager;
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

#[derive(Default)]
pub struct CommandHandler {
    /// Host name is the first key, command id is the second key.
    commands: HashMap<String, HashMap<String, Command>>,
    /// For connector communication.
    request_sender: Option<Sender<ConnectorRequest>>,
    /// Channel to send state updates to HostManager.
    state_update_sender: Option<Sender<StateUpdateMessage>>,
    host_manager: Rc<RefCell<HostManager>>,
    /// Preferences from config file.
    preferences: Preferences,
    /// Every execution gets an invocation ID. Valid ID numbers begin from 1.
    invocation_id_counter: u64,
}

impl CommandHandler {
    pub fn new(preferences: &Preferences, request_sender: Sender<ConnectorRequest>, host_manager: Rc<RefCell<HostManager>>) -> Self {
        CommandHandler {
            commands: HashMap::new(),
            request_sender: Some(request_sender),
            host_manager: host_manager.clone(),
            state_update_sender: Some(host_manager.borrow().new_state_update_sender()),
            preferences: preferences.clone(),
            invocation_id_counter: 0,
        }
    }

    pub fn add_command(&mut self, host_id: &String, command: Command) {
        if !self.commands.contains_key(host_id) {
            self.commands.insert(host_id.clone(), HashMap::new());
        }

        let command_collection = self.commands.get_mut(host_id).unwrap();
        let module_spec = command.get_module_spec();

        // Only add if missing.
        if !command_collection.contains_key(&module_spec.id) {
            log::debug!("[{}] Adding command {}", host_id, module_spec.id);
            command_collection.insert(module_spec.id, command);
        }
    }

    pub fn execute(&mut self, host_id: String, command_id: String, parameters: Vec<String>) -> u64 {
        self.invocation_id_counter += 1;

        let host = self.host_manager.borrow().get_host(&host_id);

        if host.platform.is_set() {
            log::warn!("[{}] Executing command \"{}\" despite missing platform info", host_id, command_id);
        }

        let command = self.commands.get(&host_id).unwrap()
                                   .get(&command_id).unwrap();
        let state_update_sender = self.state_update_sender.as_ref().unwrap().clone();

        // Only one of these should be implemented, but it doesn't matter if both are.
        let messages = [
            command.get_connector_messages(host.clone(), parameters.clone()),
            vec![command.get_connector_message(host.clone(), parameters)]
        ].concat();

        self.request_sender.as_ref().unwrap().send(ConnectorRequest {
            connector_spec: command.get_connector_spec(),
            source_id: command.get_module_spec().id,
            host: host.clone(),
            request_type: RequestType::Command,
            messages: messages,
            response_handler: Self::get_response_handler(host, command.box_clone(), self.invocation_id_counter, state_update_sender),
            cache_policy: CachePolicy::BypassCache, 
        }).unwrap_or_else(|error| {
            log::error!("Couldn't send message to connector: {}", error);
        });

        return self.invocation_id_counter
    }

    // Return value contains host's commands. `parameters` is not set since provided by data point later on.
    pub fn get_commands_for_host(&self, host_id: String) -> HashMap<String, CommandData> {
        if let Some(command_collection) = self.commands.get(&host_id) {
            command_collection.iter().map(|(command_id, command)| {
                (command_id.clone(), CommandData::new(command_id.clone(), command.get_display_options()))
            }).collect()
        }
        else {
            HashMap::new()
        }
    }

    pub fn get_command_for_host(&self, host_id: &String, command_id: &String) -> CommandData {
        let command_collection = self.commands.get(host_id).unwrap();
        let command = command_collection.get(command_id).unwrap();
        CommandData::new(command_id.clone(), command.get_display_options())
    }

    fn get_response_handler(host: Host, command: Command, invocation_id: u64, state_update_sender: Sender<StateUpdateMessage>) -> ResponseHandlerCallback {
        Box::new(move |results| {
            let results_len = results.len();
            let (responses, _errors): (Vec<_>, Vec<_>) =  results.into_iter().partition(Result::is_ok);
            let responses = responses.into_iter().map(Result::unwrap).collect::<Vec<_>>();
            // let _errors = errors.into_iter().map(Result::unwrap_err).collect::<Vec<_>>();

            let command_result = if results_len > 1 && responses.len() > 0 {
                command.process_responses(host.clone(), responses)
            }
            else if results_len == 1 && responses.len() == 1 {
                command.process_response(host.clone(), responses.first().unwrap())
            }
            else {
                Err(String::from("No response received"))
            };

            let result = match command_result {
                Ok(mut result) => {
                    log::debug!("[{}] Command result received: {}", host.name, result.message);
                    result.invocation_id = invocation_id;
                    result.command_id = command.get_module_spec().id;
                    result
                },
                Err(error) => {
                    log::error!("[{}] Error from command: {}", host.name, error);
                    CommandResult::new_critical_error(error)
                }
            };

            state_update_sender.send(StateUpdateMessage {
                host_name: host.name,
                display_options: command.get_display_options(),
                module_spec: command.get_module_spec(),
                data_point: None,
                command_result: Some(result),
                exit_thread: false,
            }).unwrap_or_else(|error| {
                log::error!("Couldn't send message to state manager: {}", error);
            });
        })

    }

    //
    // INTEGRATED COMMANDS
    //

    pub fn open_terminal(&self, args: Vec<String>) {
        // TODO: other kind of terminals too
        // TODO: integrated interactive terminals with ssh2::request_pty() / shell?

        let mut command_args = self.preferences.terminal_args.clone();
        command_args.extend(args);

        log::debug!("Starting local process: {} {}", self.preferences.terminal, command_args.join(" "));
        process::Command::new(self.preferences.terminal.as_str()).args(command_args).output()
                         .expect("Running command failed");
    }

    pub fn open_text_editor(&self, host_id: String, command_id: String, remote_file_path: String) {
        let host = self.host_manager.borrow().get_host(&host_id);
        let command = self.commands.get(&host_id).unwrap()
                                   .get(&command_id).unwrap();

        // Only one of these should be implemented, but it doesn't matter if both are.
        let connector_messages = [
            command.get_connector_messages(host.clone(), vec![remote_file_path.clone()]),
            vec![command.get_connector_message(host.clone(), vec![remote_file_path])]
        ].concat();

        if self.preferences.use_remote_editor {
            let mut command_args = self.preferences.terminal_args.clone();
            command_args.extend(vec![
                String::from("ssh"),
                String::from("-t"),
                host.name.clone(),
            ]);


            if self.preferences.sudo_remote_editor {
                command_args.push(String::from("sudo"));
            }

            command_args.push(self.preferences.remote_text_editor.clone());
            command_args.push(connector_messages.join(" "));

            log::debug!("Starting local process: {} {}", self.preferences.terminal, command_args.join(" "));
            process::Command::new(self.preferences.terminal.as_str()).args(command_args).output()
                                .expect("Running command failed");

            self.state_update_sender.as_ref().unwrap().send(StateUpdateMessage {
                host_name: host.name,
                display_options: command.get_display_options(),
                module_spec: command.get_module_spec(),
                data_point: None,
                command_result: Some(CommandResult::new(String::from("Successfully modified file"))),
                exit_thread: false,
            }).unwrap_or_else(|error| {
                log::error!("Couldn't send message to state manager: {}", error);
            });
        }
        else {
            self.request_sender.as_ref().unwrap().send(ConnectorRequest {
                connector_spec: command.get_connector_spec(),
                source_id: command.get_module_spec().id,
                host: host.clone(),
                request_type: RequestType::Download,
                messages: connector_messages,
                response_handler: Self::get_response_handler_text_editor(
                    host, command.box_clone(), self.preferences.clone(),
                    self.request_sender.as_ref().unwrap().clone(), self.state_update_sender.as_ref().unwrap().clone()
                ),
                cache_policy: CachePolicy::BypassCache,
            }).unwrap_or_else(|error| {
                log::error!("Couldn't send message to connector: {}", error);
            });
        }

    }

    fn get_response_handler_text_editor(host: Host, command: Command,
                                        preferences: Preferences,
                                        request_sender: Sender<ConnectorRequest>,
                                        state_update_sender: Sender<StateUpdateMessage>) -> ResponseHandlerCallback {

        Box::new(move |responses| {
            // TODO: Commands don't yet support multiple commands per module. Implement later (take a look at monitor_manager.rs).
            let response = responses.first().unwrap();

            match response {
                Ok(response_message) => {
                    // TODO: check that destination file hasn't changed.

                    log::debug!("Starting local process: {} {}", preferences.text_editor, response_message.message);
                    process::Command::new(preferences.text_editor).args(vec![response_message.message.clone()]).output()
                                        .expect("Running command failed");

                    request_sender.send(ConnectorRequest {
                        connector_spec: command.get_connector_spec(),
                        source_id: command.get_module_spec().id,
                        host: host.clone(),
                        request_type: RequestType::Upload,
                        messages: vec![response_message.message.clone()],
                        response_handler: Self::get_response_handler_upload_file(host, command, state_update_sender),
                        cache_policy: CachePolicy::BypassCache,
                    }).unwrap_or_else(|error| {
                        log::error!("Couldn't send message to connector: {}", error);
                    });
                },
                Err(error) => {
                    log::error!("Error downloading file: {}", error);
                    state_update_sender.send(StateUpdateMessage {
                        host_name: host.name,
                        display_options: command.get_display_options(),
                        module_spec: command.get_module_spec(),
                        data_point: None,
                        command_result: Some(CommandResult::new_critical_error(error.clone())),
                        exit_thread: false,
                    }).unwrap_or_else(|error| {
                        log::error!("Couldn't send message to state manager: {}", error);
                    });
                }
            };
        })
    }

    fn get_response_handler_upload_file(host: Host, command: Command, state_update_sender: Sender<StateUpdateMessage>) -> ResponseHandlerCallback {

        Box::new(move |responses| {
            // TODO: Commands don't yet support multiple commands per module. Implement later (take a look at monitor_manager.rs).
            let response = responses.first().unwrap();

            let command_result = match response {
                Ok(_) => {
                    CommandResult::new(String::from("File updated successfully"))
                },
                Err(error) => {
                    log::error!("Error uploading file: {}", error);
                    CommandResult::new_critical_error(error.clone())
                }
            };

            state_update_sender.send(StateUpdateMessage {
                host_name: host.name,
                display_options: command.get_display_options(),
                module_spec: command.get_module_spec(),
                data_point: None,
                command_result: Some(command_result),
                exit_thread: false,
            }).unwrap_or_else(|error| {
                log::error!("Couldn't send message to state manager: {}", error);
            });
        })
    }
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
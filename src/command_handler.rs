
use std::sync::mpsc::Sender;
use std::collections::HashMap;
use serde_derive::Serialize;
use std::process;

use crate::{
    Host,
    host_manager::StateUpdateMessage,
    connection_manager::ConnectorRequest, 
    connection_manager::ResponseHandlerCallback,
    frontend::DisplayOptions,
};

use crate::module::{
    command::Command,
    command::CommandResult,
};

#[derive(Default)]
pub struct CommandHandler {
    // Command id is the second key.
    commands: HashMap<Host, HashMap<String, Command>>,
    // For connector communication.
    request_sender: Option<Sender<ConnectorRequest>>,
    // Channel to send state updates to HostManager.
    state_update_sender: Option<Sender<StateUpdateMessage>>,

    // Every execution gets an invocation id. Valid id numbers begin from 1.
    invocation_id_counter: u64,
}

impl CommandHandler {
    pub fn new(request_sender: Sender<ConnectorRequest>, state_update_sender: Sender<StateUpdateMessage>) -> Self {
        CommandHandler {
            commands: HashMap::new(),
            request_sender: Some(request_sender),
            state_update_sender: Some(state_update_sender),
            invocation_id_counter: 0,
        }
    }

    pub fn add_command(&mut self, host: &Host, command: Command) {
        if !self.commands.contains_key(host) {
            self.commands.insert(host.clone(), HashMap::new());
        }

        let command_collection = self.commands.get_mut(host).unwrap();
        let module_spec = command.get_module_spec();

        // Only add if missing.
        if !command_collection.contains_key(&module_spec.id) {
            log::debug!("Adding command {}", module_spec.id);
            command_collection.insert(module_spec.id, command);
        }
    }

    pub fn execute(&mut self, host_id: String, command_id: String, target_id: String) -> u64 {
        // TODO: better solution for searching?
        let (host, command_collection) = self.commands.iter().find(|(host, _)| host.name == host_id).unwrap();
        let command = command_collection.get(&command_id).unwrap();

        let state_update_sender = self.state_update_sender.as_ref().unwrap().clone();
        self.invocation_id_counter += 1;

        self.request_sender.as_ref().unwrap().send(ConnectorRequest {
            connector_id: command.get_connector_spec().unwrap().id,
            source_id: command.get_module_spec().id,
            host: host.clone(),
            message: command.get_connector_request(target_id),
            response_handler: Self::get_response_handler(host.clone(), command.clone_module(), self.invocation_id_counter, state_update_sender),
        }).unwrap_or_else(|error| {
            log::error!("Couldn't send message to connector: {}", error);
        });

        return self.invocation_id_counter
    }

    pub fn open_terminal(&self, args: Vec<String>) {
        // TODO: other kind of terminals too
        // TODO: integrated interactive terminals with ssh2::request_pty() / shell?

        let mut command_args = vec![String::from("-e")];
        command_args.extend(args);

        log::debug!("Starting process: /usr/bin/konsole {}", command_args.join(" "));
        process::Command::new("/usr/bin/konsole")
                         .args(command_args)
                         .output().expect("Running command failed");
    }

    // Return value contains host's commands and command parameters as strings.
    pub fn get_host_commands(&self, host_id: String) -> HashMap<String, CommandData> {
        if let Some((_, command_collection)) = self.commands.iter().find(|(host, _)| host.name == host_id) {
            command_collection.iter().map(|(command_id, command)| {
                (command_id.clone(), CommandData::new(command_id.clone(), command.get_display_options()))
            }).collect()
        }
        else {
            HashMap::new()
        }
    }

    pub fn get_host_command(&self, host_id: String, command_id: String) -> CommandData {
        let (_, command_collection) = self.commands.iter().find(|(host, _)| host.name == host_id).unwrap();
        let command = command_collection.get(&command_id).unwrap();
        CommandData::new(command_id, command.get_display_options())
    }

    fn get_response_handler(host: Host, command: Command, invocation_id: u64, state_update_sender: Sender<StateUpdateMessage>) -> ResponseHandlerCallback {
        Box::new(move |response, _connector_is_connected| {
            let command_result = match response {
                Err(error) => {
                    log::error!("Error sending command: {}", error);
                    Some(CommandResult::new_critical_error(error))
                },
                Ok(response) => {
                    match command.process_response(&response) {
                        Ok(mut result) => {
                            log::debug!("Command result received: {}", result.message);
                            result.invocation_id = invocation_id;
                            Some(result)
                        },
                        Err(error) => {
                            log::error!("Error from command: {}", error);
                            Some(CommandResult::new_critical_error(error))
                        }
                    }
                }
            };

            state_update_sender.send(StateUpdateMessage {
                host_name: host.name,
                display_options: command.get_display_options(),
                module_spec: command.get_module_spec(),
                data_point: None,
                command_result: command_result,
            }).unwrap_or_else(|error| {
                log::error!("Couldn't send message to state manager: {}", error);
            });
        })

    }

}


#[derive(Clone, Serialize)]
pub struct CommandData {
    pub command_id: String,
    pub display_options: DisplayOptions,
}

impl CommandData {
    pub fn new(command_id: String, display_options: DisplayOptions) -> Self {
        CommandData {
            command_id: command_id,
            display_options: display_options,
        }
    }
}
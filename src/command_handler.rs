
use std::sync::mpsc::Sender;

use crate::Host;
use crate::module::command::Command;
use crate::module::command::CommandResult;
use crate::host_manager::StateUpdateMessage;
use crate::connection_manager::ConnectorRequest;
use crate::module::monitoring::DisplayOptions;

pub struct CommandHandler {
    request_sender: Sender<ConnectorRequest>,
    state_update_sender: Sender<StateUpdateMessage>,
}

impl CommandHandler {
    pub fn new(request_sender: Sender<ConnectorRequest>, state_update_sender: Sender<StateUpdateMessage>) -> Self {
        CommandHandler {
            request_sender: request_sender,
            state_update_sender: state_update_sender,
        }
    }

    pub fn execute(&self, host: Host, command: Command) {
        let state_update_sender = self.state_update_sender.clone();

        self.request_sender.send(ConnectorRequest {
            connector_id: command.get_connector_spec().unwrap().id,
            source_id: command.get_module_spec().id,
            host: host.clone(),
            message: command.get_connector_request(None),

            response_handler: Box::new(move |output, _connector_is_connected| {
                let response_result = match command.process_response(&output) {
                    Ok(result) => {
                        log::debug!("Command result received: {}", result.message);
                        result
                    },
                    Err(error) => {
                        log::error!("Error from command: {}", error);
                        CommandResult::empty_and_critical()
                    }
                };

                state_update_sender.send(StateUpdateMessage {
                    host_name: host.name,
                    display_options: DisplayOptions::just_style(crate::module::monitoring::DisplayStyle::CriticalityLevel),
                    module_spec: command.get_module_spec(),
                    data_point: None,
                    command_result: Some(response_result),
                }).unwrap_or_else(|error| {
                    log::error!("Couldn't send message to state manager: {}", error);
                });
            })
        }).unwrap_or_else(|error| {
            log::error!("Couldn't send message to connector: {}", error);
        });
    }

}
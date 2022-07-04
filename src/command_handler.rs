
use std::sync::mpsc::Sender;
use std::collections::HashMap;

use crate::connection_manager::ConnectorRequest;
use crate::host_manager::{ HostManager, StateUpdateMessage };
use crate::module::{
    module_factory::ModuleFactory,
    command::Command,
    command::CommandResult,
    ModuleSpecification,
};

#[derive(Default)]
pub struct CommandHandler {
    request_sender: Option<Sender<ConnectorRequest>>,
    state_update_sender: Option<Sender<StateUpdateMessage>>,
    host_manager: Option<HostManager>,
    module_factory: Option<ModuleFactory>,
}

impl CommandHandler {
    pub fn new(request_sender: Sender<ConnectorRequest>, host_manager: HostManager, module_factory: ModuleFactory) -> Self {
        CommandHandler {
            request_sender: Some(request_sender),
            state_update_sender: Some(host_manager.new_state_update_sender()),
            host_manager: Some(host_manager),
            module_factory: Some(module_factory),
        }
    }

    pub fn execute(&mut self, host_id: String, command_id: String) {
        let host = self.host_manager.as_mut().unwrap().get_host(&host_id);
        let command = self.module_factory.as_ref().unwrap().new_command(&ModuleSpecification::new(&command_id, "0.0.1"), &HashMap::new());

        let state_update_sender = self.state_update_sender.clone();

        self.request_sender.as_ref().unwrap().send(ConnectorRequest {
            connector_id: command.get_connector_spec().unwrap().id,
            source_id: command.get_module_spec().id,
            host: host.clone(),
            message: command.get_connector_request(None),

            response_handler: Box::new(move |output, _connector_is_connected| {
                let command_result = match command.process_response(&output) {
                    Ok(result) => {
                        log::debug!("Command result received: {}", result.message);
                        result
                    },
                    Err(error) => {
                        log::error!("Error from command: {}", error);
                        CommandResult::empty_and_critical()
                    }
                };

                state_update_sender.unwrap().send(StateUpdateMessage {
                    host_name: host.name,
                    display_options: command.get_display_options(),
                    module_spec: command.get_module_spec(),
                    data_point: None,
                    command_result: Some(command_result),
                }).unwrap_or_else(|error| {
                    log::error!("Couldn't send message to state manager: {}", error);
                });
            })
        }).unwrap_or_else(|error| {
            log::error!("Couldn't send message to connector: {}", error);
        });
    }

}
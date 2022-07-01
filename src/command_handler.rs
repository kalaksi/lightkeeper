
use std::sync::mpsc::Sender;
use crate::module::monitoring::{DataPoint, Criticality};

use crate::Host;
use crate::module::command::Command;
use crate::host_manager::DataPointMessage;
use crate::connection_manager::ConnectorRequest;
use crate::module::monitoring::DisplayOptions;

pub struct CommandHandler {
    request_sender: Sender<ConnectorRequest>,
    state_update_sender: Sender<DataPointMessage>,
}

impl CommandHandler {
    pub fn new(request_sender: Sender<ConnectorRequest>, state_update_sender: Sender<DataPointMessage>) -> Self {
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
                log::debug!("{}", output);
                let response_result = command.process_response(&output);
                let send_result = state_update_sender.send(DataPointMessage {
                    host_name: host.name,
                    display_options: DisplayOptions::just_style(crate::module::monitoring::DisplayStyle::CriticalityLevel),
                    module_spec: command.get_module_spec(),
                    data_point: DataPoint::new_with_level(String::from("test"), Criticality::Critical),
                });
            })
        }).unwrap_or_else(|error| {
            log::error!("Couldn't send message to connector: {}", error);
        });
    }

}
use std::collections::HashMap;
use crate::frontend;
use crate::host::Host;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("linux-packages-update", "0.0.1")]
pub struct Update;

impl Module for Update {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Self { }
    }
}

impl CommandModule for Update {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("packages"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from(""),
            display_text: String::from("Update packages"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, _host: Host, _parameters: Vec<String>) -> String {
        todo!()
    }

    fn process_response(&self, _host: Host, _response: &ResponseMessage) -> Result<CommandResult, String> {
        todo!()
    }
}
use std::collections::HashMap;
use crate::frontend;
use crate::utils::enums::Criticality;
use crate::module::connection::ResponseMessage;
use crate::module::{
    Module,
    command::CommandModule,
    command::Command,
    command::CommandResult,
    Metadata,
    ModuleSpecification,
};


#[derive(Clone)]
pub struct Shutdown;

impl Module for Shutdown {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("shutdown", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Shutdown { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl CommandModule for Shutdown {
    fn clone_module(&self) -> Command {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("host"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("shutdown"),
            display_text: String::from("Shut down"),
            confirmation_text: String::from("Really shut down host?"),
            ..Default::default()
        }
    }

    fn get_connector_request(&self, _parameters: Vec<String>) -> String {
        String::from("sudo poweroff")
    }

    fn process_response(&self, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            Ok(CommandResult::new_with_level(response.message.clone(), Criticality::Warning))
        }
        else {
            Ok(CommandResult::new(response.message.clone()))
        }
    }
}
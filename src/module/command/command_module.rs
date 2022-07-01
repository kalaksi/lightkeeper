
use std::collections::HashMap;
use crate::module::{ Module, ModuleSpecification };

pub type Command = Box<dyn CommandModule + Send + Sync>;

pub trait CommandModule : Module {
    fn new_command_module(settings: &HashMap<String, String>) -> Command where Self: Sized + 'static + Send + Sync {
        Box::new(Self::new(settings))
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        None
    }

    fn get_subcommands(&self) -> Option<Vec<String>> {
        None
    }

    fn get_connector_request(&self, _subcommand: Option<String>) -> String {
        String::from("")
    }

    fn process_response(&self, response: &String) -> Result<String, String>;

}
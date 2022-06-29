
use std::collections::HashMap;
use crate::Host;
use crate::module::{ Module, ModuleSpecification };

pub trait CommandModule : Module {
    fn new_command_module(settings: &HashMap<String, String>) -> Box<dyn CommandModule> where Self: Sized + 'static {
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

    fn process_response(&self, host: &Host, response: &String) -> Result<String, String>;

}
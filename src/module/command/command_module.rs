
use std::collections::HashMap;
use crate::module::{ Module, connection::ConnectionModule };

pub trait CommandModule : Module {
    fn execute(&self, connection: &mut dyn ConnectionModule) -> Result<String, String>;

    fn new_command_module(settings: &HashMap<String, String>) -> Box<dyn CommandModule> where Self: Sized + 'static {
        Box::new(Self::new(settings))
    }
}
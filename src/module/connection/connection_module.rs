use std::net::SocketAddr;
use crate::module::{
    module::Module,
    ModuleSpecification,
    connection::AuthenticationDetails,
};

pub trait ConnectionModule : Module {
    fn connect(&mut self, address: &SocketAddr, authentication: Option<AuthenticationDetails>) -> Result<(), String>;
    fn send_message(&self, message: &str) -> Result<String, String>;

    fn new_connection_module() -> Box<dyn ConnectionModule> where Self: Sized + 'static {
        Box::new(Self::new())
    }
}
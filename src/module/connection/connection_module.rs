use crate::module::{
    module::Module,
    connection::AuthenticationDetails,
};

use std::net::SocketAddr;

pub trait ConnectionModule : Module {
    fn connect(&mut self, address: &SocketAddr, authentication: Option<AuthenticationDetails>) -> Result<(), String>;
    fn send_message(&self, message: &str) -> Result<String, String>;

    fn new_connection_module() -> Box<dyn ConnectionModule> where Self: Sized + 'static {
        Box::new(Self::new())
    }
}
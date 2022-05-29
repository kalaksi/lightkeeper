
use std::net::SocketAddr;
use crate::module::{
    module::Module,
    metadata::Metadata,
    connection::ConnectionModule,
    connection::AuthenticationDetails,
    ModuleSpecification,
};

pub struct Empty { 
}

impl Module for Empty {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new(String::from(""), String::from("")),
            display_name: String::from(""),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new() -> Self {
        Empty { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl ConnectionModule for Empty {
    fn connect(&mut self, address: &SocketAddr, authentication: Option<AuthenticationDetails>) -> Result<(), String> {
        Ok(())
    }

    fn send_message(&self, message: &str) -> Result<String, String>
    {
        Ok(String::new())
    }

}


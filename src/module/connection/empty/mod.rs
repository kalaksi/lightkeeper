
use std::{ collections::HashMap, net::IpAddr };
use crate::module::{
    Module,
    metadata::Metadata,
    connection::ConnectionModule,
    ModuleSpecification,
};

pub struct Empty { 
}

impl Module for Empty {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new(String::from(""), String::from("")),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Empty { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl ConnectionModule for Empty {
    fn connect(&mut self, _address: &IpAddr) -> Result<(), String> {
        panic!("No connect() implemented for empty connector");
    }

    fn send_message(&self, _message: &str) -> Result<String, String> {
        panic!("No send_message() implemented for empty connector");
    }
}



use crate::Host;
use crate::module::{
    Module,
    Metadata,
    connection::ConnectionModule,
    monitoring::{MonitoringModule, MonitoringData},
    ModuleSpecification,
};

pub struct Ssh;

impl Module for Ssh {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new(String::from("ssh"), String::from("0.0.1")),
            display_name: String::from("SSH"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new() -> Self {
        Ssh { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Ssh {
    fn get_connector_spec(&self) -> ModuleSpecification {
        ModuleSpecification::new(String::from("ssh"), String::from("0.0.1"))
    }

    fn refresh(&self, _host: &Host, connection: &mut Box<dyn ConnectionModule>) -> Result<MonitoringData, String> {
        match &connection.is_connected() {
            true => Ok(MonitoringData::new(String::from("up"), String::from(""))),
            false => Ok(MonitoringData::new(String::from("down"), String::from(""))),
        }
    }
}
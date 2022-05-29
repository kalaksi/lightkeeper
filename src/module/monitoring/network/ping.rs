
use oping;
use crate::Host;
use crate::module::{
    Module,
    Metadata,
    connection::ConnectionModule,
    monitoring::MonitoringModule,
    monitoring::MonitoringData,
    ModuleSpecification,
};


pub struct Ping;

impl Module for Ping {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new(String::from("ping"), String::from("0.0.1")),
            display_name: String::from("Ping"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new() -> Self {
        Ping { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Ping {
    fn refresh(&self, host: &Host, _connection: &mut Box<dyn ConnectionModule>) -> Result<MonitoringData, String> {
        let mut ping = oping::Ping::new();
        ping.set_timeout(5.0)
            .map_err(|e| e.to_string())?;
        ping.add_host(host.socket_address.ip().to_string().as_str())
            .map_err(|e| e.to_string())?;

        let mut responses = ping.send()
                                .map_err(|e| e.to_string())?;

        let response = responses.next().unwrap();
        Ok(MonitoringData::new(response.latency_ms.to_string(), String::from("ms")))
    }
}
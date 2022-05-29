use chrono::{ DateTime, Utc };

use crate::module::{
    module::Module,
    ModuleSpecification,
    connection::ConnectionModule
};

pub trait MonitoringModule : Module {
    fn refresh(&self, connection: &mut Box<dyn ConnectionModule>) -> Result<MonitoringData, String>;
    fn get_connector_spec(&self) -> ModuleSpecification;

    fn new_monitoring_module() -> Box<dyn MonitoringModule> where Self: Sized + 'static {
        Box::new(Self::new())
    }

}

pub struct MonitoringData {
    pub value: String,
    pub unit: String,
    // TODO: check how memory hungry this type is
    pub time: DateTime<Utc>,
}

impl MonitoringData {
    pub fn new(value: String, unit: String) -> Self {
        MonitoringData {
            value: value,
            unit: unit,
            time: Utc::now(),
        }
    }
}

impl Default for MonitoringData {
    fn default() -> Self {
        MonitoringData {
            value: String::new(),
            unit: String::new(),
            time: Utc::now(),
        }
    }
}

use std::time::Duration;

use crate::module::{
    module::Module,
    connection::ConnectionModule
};

pub trait MonitoringModule : Module {
    fn refresh(&self, connection: &mut Box<dyn ConnectionModule>) -> Result<MonitoringData, String>;

    fn new_monitoring_module() -> Box<dyn MonitoringModule> where Self: Sized + 'static {
        Box::new(Self::new())
    }
}

pub struct MonitoringData {
    pub value: String,
    pub unit: String,
    pub retention: Duration,
}

impl Default for MonitoringData {
    fn default() -> Self {
        MonitoringData {
            value: String::new(),
            unit: String::new(),
            retention: Duration::from_secs(0),
        }
    }
}

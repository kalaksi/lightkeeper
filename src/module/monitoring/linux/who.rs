use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
    utils::string_manipulation,
};
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;


#[monitoring_module("who", "0.0.1")]
pub struct Who;

impl Module for Who {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Who { }
    }
}

impl MonitoringModule for Who {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("User sessions"),
            category: String::from("users"),
            use_multivalue: true,
            ignore_from_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _parent_result: DataPoint) -> String {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            String::from("who -s")
        }
        else {
            String::new()
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _parent_result: DataPoint) -> Result<DataPoint, String> {
        let mut result = DataPoint::empty();

        let lines = response.message.split('\n');
        for line in lines {
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split_whitespace();
            let user = parts.next().unwrap().to_string();
            let _pts = parts.next().unwrap().to_string();
            // TODO: format according to locales.
            let login_date = parts.next().unwrap().to_string();
            let login_time = parts.next().unwrap().to_string();
            // Removes parentheses.
            let ip_address = string_manipulation::get_string_between(&parts.next().unwrap(), "(", ")");
            
            let value_text = format!("{} {} (from {})", login_date, login_time, ip_address);
            let data_point = DataPoint::labeled_value(user, value_text);
            result.multivalue.push(data_point);
        }

        if result.multivalue.is_empty() {
            result.multivalue.push(DataPoint::labeled_value("No users logged in", " "));
        }

        Ok(result)
    }
}
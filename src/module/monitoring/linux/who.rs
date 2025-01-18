use std::collections::HashMap;
use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
    utils::string_manipulation,
};
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;


#[monitoring_module(
    name="who",
    version="0.0.1",
    description="Gets list of logged in users. Useful if there's a chance someone else is operating the server at the same time.",
)]
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
            display_text: String::from("Login sessions"),
            category: String::from("host"),
            use_multivalue: true,
            use_without_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _parent_result: DataPoint) -> Result<String, LkError> {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(String::from("who -s"))
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _parent_result: DataPoint) -> Result<DataPoint, String> {
        if response.is_error() {
            return Err(response.message);
        }

        let mut result = DataPoint::empty();

        let lines = response.message.lines().filter(|line| !line.is_empty());
        for line in lines {
            let parts = line.split_whitespace().collect::<Vec<&str>>();
            if parts.len() < 4 {
                return Ok(DataPoint::invalid_response());
            }

            let user = parts[0].to_string();
            // let _pts = parts[1].to_string();
            // TODO: format according to locales.
            let login_date = parts[2].to_string();
            let login_time = parts[3].to_string();

            let value_text = if parts.len() > 4 {
                // Removes parentheses.
                let ip_address = string_manipulation::get_string_between(&parts[4], "(", ")");
                format!("{} {} (from {})", login_date, login_time, ip_address)
            }
            else {
                format!("{} {}", login_date, login_time)
            };
            
            let data_point = DataPoint::labeled_value(user, value_text);
            result.multivalue.push(data_point);
        }

        if result.multivalue.is_empty() {
            result.multivalue.push(DataPoint::labeled_value("No users logged in", " "));
        }

        Ok(result)
    }
}
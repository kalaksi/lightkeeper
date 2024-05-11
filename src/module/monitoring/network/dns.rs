
use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::utils::ShellCommand;
use crate::{
    Host,
    frontend,
};

use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="network-dns",
    version="0.0.1",
    description="Provides information about DNS settings.",
    settings={
    }
)]
pub struct Dns {
}

impl Module for Dns {
    fn new(_: &HashMap<String, String>) -> Self {
        Dns {
        }
    }
}

impl MonitoringModule for Dns {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("DNS settings"),
            category: String::from("network"),
            use_multivalue: true,
            ignore_from_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_messages(&self, host: Host, _parent_result: DataPoint) -> Result<Vec<String>, String> {
        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "10") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {

            let command_resolvconf = ShellCommand::new_from(vec!["grep", "-E", "^nameserver", "/etc/resolv.conf"]);
            // TODO: use busctl instead?
            let mut command_resolvectl = ShellCommand::new_from(vec!["resolvectl", "dns"]);
            // Most likely means that resolved isn't available so quietly ignoring errors.
            command_resolvectl.ignore_error = true;

            Ok(vec![command_resolvconf.to_string(), command_resolvectl.to_string()])
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_responses(&self, _host: Host, responses: Vec<ResponseMessage>, _parent_result: DataPoint) -> Result<DataPoint, String> {
        if responses.is_empty() {
            return Ok(DataPoint::empty());
        }

        let mut result = DataPoint::empty();


        let resolvconf_response = &responses[0];
        if resolvconf_response.is_success() {
            for line in resolvconf_response.message.lines() {
                let mut parts = line.split(' ');
                let dns_server = parts.nth(1).unwrap_or_default().trim().to_string();

                let mut datapoint = DataPoint::label(dns_server);
                datapoint.description = String::from("resolv.conf");
                datapoint.is_from_cache = resolvconf_response.is_from_cache;
                result.multivalue.push(datapoint);
            }
        }

        let resolvectl_response = &responses[1];
        if !resolvectl_response.message.contains(".service not found") {
            for line in resolvectl_response.message.lines() {
                if line.starts_with("Link") {
                    let mut parts = line.split("): ");
                    let dns_servers = parts.nth(1).unwrap_or_default()
                                        .split_whitespace();

                    for dns_server in dns_servers {
                        let mut datapoint = DataPoint::label(dns_server);
                        datapoint.description = String::from("systemd-resolved");
                        datapoint.is_from_cache = resolvectl_response.is_from_cache;
                        result.multivalue.push(datapoint);
                    }
                }
            }
        }

        Ok(result)
    }
}
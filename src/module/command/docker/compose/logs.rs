use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="docker-compose-logs",
    version="0.0.1",
    description="Show docker-compose logs for services.",
)]
pub struct Logs {
}

impl Module for Logs {
    fn new(_settings: &HashMap<String, String>) -> Logs {
        Logs {
        }
    }
}

impl CommandModule for Logs {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-compose"),
            parent_id: String::from("docker-compose"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("view-document"),
            display_text: String::from("Show logs"),
            action: UIAction::LogView,
            multivalue_level: 2,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, String> {
        let compose_file = parameters.first().unwrap();
        // let project = parameters.get(1).unwrap();
        let service_name = parameters.get(2).unwrap();
        let page_number = parameters.get(3).unwrap_or(&String::from("1")).parse::<i32>().unwrap();
        let page_size = parameters.get(4).unwrap_or(&String::from("400")).parse::<i32>().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if host.platform.version_is_same_or_greater_than(platform_info::Flavor::Debian, "8") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::Ubuntu, "20") {

            // TODO: Don't hardcode page size

            if page_number > 1 {
                let row_count = page_number * page_size;
                command.arguments(vec!["docker-compose", "-f", compose_file, "logs", "--tail", &row_count.to_string(), "--no-color", "-t", service_name]);
                // would be nice to return just the needed parts, but tailing will possibly return different rows,
                // so currently just returning everything
                    // .pipe_to(vec!["head", "-n", &page_size.to_string()]);
            }
            else {
                command.arguments(vec!["docker-compose", "-f", compose_file, "logs", "--tail", &page_size.to_string(), "--no-color", "-t", service_name]);
            }
        }
        else if host.platform.version_is_same_or_greater_than(platform_info::Flavor::RedHat, "8") ||
                host.platform.version_is_same_or_greater_than(platform_info::Flavor::CentOS, "8") {

            command.arguments(vec!["docker", "compose", "-f", compose_file, "logs", "--tail", "400", "--no-color", "-t", service_name]);
        }
        else {
            return Err(String::from("Unsupported platform"));
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &connection::ResponseMessage) -> Result<CommandResult, String> {
        // Removes the prefix "PROJECT_NAME_1             |"
        let prefix_removed = response.message.lines().map(|line| {
            line.split_once("|").map(|(_, rest)| rest.trim_start()).unwrap_or(line)
        }).collect::<Vec<&str>>().join("\n");

        if response.is_error() {
            return Err(response.message.clone());
        }
        Ok(CommandResult::new_hidden(prefix_removed))
    }
}
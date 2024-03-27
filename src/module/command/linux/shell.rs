use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::enums;
use lightkeeper_module::command_module;

#[command_module(
    name="linux-shell",
    version="0.0.1",
    description="Opens a shell on a Linux host.",
    settings={
        as_root => "Switch to root user after login. Default: true.",
    }
)]
pub struct Shell {
    pub as_root: bool,
}

impl Module for Shell {
    fn new(settings: &HashMap<String, String>) -> Self {
        Shell {
            as_root: settings.get("as_root").unwrap_or(&String::from("true")).parse::<bool>().unwrap_or(true),
        }
    }
}

impl CommandModule for Shell {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("host"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("terminal"),
            display_text: String::from("Open shell"),
            depends_on_criticality: vec![enums::Criticality::Normal],
            action: UIAction::Terminal,
            tab_title: String::from("Host shell"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> Result<String, String> {
        let mut command = ShellCommand::new();
        command.use_sudo = false;

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "4") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "16") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "4") ||
           host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "4") {

            if self.as_root {
                if host.settings.contains(&crate::host::HostSetting::UseSudo) {
                    command.arguments(vec!["sudo", "-i"]);
                }
                else {
                    command.arguments(vec!["su", "-"]);
                }
            }
        }
        else {
            return Err(String::from("Unsupported platform"));
        }

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new_info(response.message.clone()))
    }
}
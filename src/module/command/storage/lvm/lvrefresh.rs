use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;


#[command_module(
    name="storage-lvm-lvrefresh",
    version="0.0.1",
    description="Refreshes an LVM logical volume.",
    settings={
    }
)]
pub struct LVRefresh {
}

impl Module for LVRefresh {
    fn new(_settings: &HashMap<String, String>) -> Self {
        LVRefresh {
        }
    }
}

impl CommandModule for LVRefresh {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("storage"),
            parent_id: String::from("storage-lvm-logical-volume"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("refresh"),
            display_text: String::from("Refresh"),
            depends_on_value: vec![String::from("Refresh needed")],
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, String> {
        let lv_path = parameters.get(0).unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if host.platform.version_is_same_or_greater_than(platform_info::Flavor::Debian, "9") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::Ubuntu, "20") {
            // TODO: centos?

            command.arguments(vec!["lvchange", "--refresh", lv_path]);
            Ok(command.to_string())
        }
        else {
            Err(String::from("Unsupported platform"))
        }

    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::new_info(String::new()))
        }
        else {
            Ok(CommandResult::new_error(response.message.clone()))
        }
    }
}
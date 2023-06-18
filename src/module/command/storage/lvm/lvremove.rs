use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;


#[command_module("storage-lvm-lvremove", "0.0.1")]
pub struct LVRemove {
}

impl Module for LVRemove {
    fn new(_settings: &HashMap<String, String>) -> Self {
        LVRemove {
        }
    }
}

impl CommandModule for LVRemove {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("storage"),
            parent_id: String::from("storage-lvm-logical-volume"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("delete"),
            display_text: String::from("Remove"),
            confirmation_text: String::from("Are you sure you want to remove this logical volume?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> String {
        let lv_path = parameters.get(0).unwrap();
        let _vg_name = parameters.get(1).unwrap();
        let _lv_name = parameters.get(2).unwrap();
        let _lv_size = parameters.get(3).unwrap();

        let mut command = ShellCommand::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            if host.platform.version_is_newer_than(platform_info::Flavor::Debian, "8") &&
               host.platform.version_is_older_than(platform_info::Flavor::Debian, "11") {
                 command.arguments(vec!["lvremove", "-y", lv_path]);
            };

            command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);
        }

        command.to_string()
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 && response.message.contains("successfully removed"){
            Ok(CommandResult::new(String::new()))
        }
        else {
            Ok(CommandResult::new_error(response.message.clone()))
        }
    }
}
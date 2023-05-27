use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::utils::string_validation;
use lightkeeper_module::command_module;


#[command_module("linux-lvm-lvresize", "0.0.1")]
pub struct LVResize {
}

impl Module for LVResize {
    fn new(_settings: &HashMap<String, String>) -> Self {
        LVResize {
        }
    }
}

impl CommandModule for LVResize {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("storage"),
            parent_id: String::from("linux-lvm-logical-volume"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("resize-column-2"),
            display_text: String::from("Resize"),
            user_parameters: vec![
                frontend::UserInputField::decimal_number_with_units("New size", "20G", vec![
                    String::from("r"), String::from("R"),
                    String::from("b"), String::from("B"),
                    String::from("s"), String::from("S"),
                    String::from("k"), String::from("K"),
                    String::from("m"), String::from("M"),
                    String::from("g"), String::from("G"),
                    String::from("t"), String::from("T"),
                    String::from("p"), String::from("P"),
                    String::from("e"), String::from("E")
                ]),
            ],
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> String {
        let lv_path = parameters.get(0).unwrap();
        let _vg_name = parameters.get(1).unwrap();
        let _lv_name = parameters.get(2).unwrap();
        let _lv_size = parameters.get(3).unwrap();
        let new_size = crate::utils::remove_whitespace(parameters.get(4).unwrap());

        if !string_validation::is_numeric_with_unit(&new_size, &self.get_display_options().user_parameters[0].units) {
            panic!("Invalid size: {}", new_size);
        }

        let mut command = ShellCommand::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            if host.platform.version_is_newer_than(platform_info::Flavor::Debian, "8") &&
               host.platform.version_is_older_than(platform_info::Flavor::Debian, "11") {
                 command.arguments(vec!["lvresize", "--size", &new_size, lv_path]);
            };

            command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);
        }

        command.to_string()
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 && response.message.contains("successfully resized"){
            Ok(CommandResult::new(String::new()))
        }
        else {
            Ok(CommandResult::new_error(response.message.clone()))
        }
    }
}
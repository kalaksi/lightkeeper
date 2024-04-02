use regex::Regex;
use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="nixos-rebuild-switch",
    version="0.0.1",
    description="Runs 'nixos-rebuild switch' and shows output",
)]
pub struct RebuildSwitch {
    regex_path_count: Regex,
}

impl Module for RebuildSwitch {
    fn new(_settings: &HashMap<String, String>) -> RebuildSwitch {
        RebuildSwitch {
            regex_path_count: Regex::new(r"(?i)these (\d+) paths will be fetched").unwrap(),
        }
    }
}

impl CommandModule for RebuildSwitch {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("nixos"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("build"),
            display_text: String::from("nixos-rebuild switch"),
            action: UIAction::FollowOutput,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> Result<String, String> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {
            command.arguments(vec!["nixos-rebuild", "switch"]); 
        }
        else {
            return Err(String::from("Unsupported platform"));
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            let mut progress: u8 = 0;
            let mut copied: u16 = 0;
            let mut to_fetch: u16 = 0;

            for line in response.message.lines() {
                if let Some(captures) = self.regex_path_count.captures(line) {
                    to_fetch = captures.get(1).unwrap().as_str().parse().unwrap();
                    progress = 10;
                }
                if line.contains("copying path '") {
                    copied += 1;
                    progress = 10 + (copied as f32 / to_fetch as f32 * 80.0) as u8;
                }
            }
            Ok(CommandResult::new_partial(response.message.clone(), progress))
        }
        else {
            if response.return_code == 0 {
                Ok(CommandResult::new_hidden(response.message.clone()))
            }
            else {
                Ok(CommandResult::new_hidden(response.message.clone())
                                .with_criticality(crate::enums::Criticality::Error))
            }
        }
    }
}
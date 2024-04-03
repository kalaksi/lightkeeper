use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="nixos-collectgarbage",
    version="0.0.1",
    description="Collects garbage from the Nix store.",
    settings={
        delete_older_than => "Delete generations of profiles older than the specified number of days. Default: 7.",
    }
)]
pub struct CollectGarbage {
    delete_older_than: u32,
}

impl Module for CollectGarbage {
    fn new(settings: &HashMap<String, String>) -> CollectGarbage {
        CollectGarbage {
            delete_older_than: settings.get("delete_older_than").unwrap_or(&String::from("7")).parse::<u32>().unwrap_or(0),
        }
    }
}

impl CommandModule for CollectGarbage {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("nixos"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("clear"),
            display_text: String::from("Delete old generations and collect garbage"),
            action: UIAction::FollowOutput,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> Result<String, String> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {
            command.arguments(vec!["nix-collect-garbage", "--delete-older-than", format!("{}d", self.delete_older_than).as_str()]); 
        }
        else {
            return Err(String::from("Unsupported platform"));
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            let progress = 10;
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
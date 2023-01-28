use std::{
    collections::HashMap,
};
use crate::frontend;
use crate::host::Host;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("docker-compose-up", "0.0.1")]
pub struct Up {
    use_sudo: bool,
}

impl Module for Up {
    fn new(settings: &HashMap<String, String>) -> Self {
        Up {
            use_sudo: settings.get("use_sudo").and_then(|value| Some(value == "true")).unwrap_or(true),
        }
    }
}

impl CommandModule for Up {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-compose"),
            parent_id: String::from("docker-compose"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("update-none"),
            display_text: String::from("Up"),
            action: UIAction::None,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, _host: Host, parameters: Vec<String>) -> String {
        let compose_file = parameters[0].clone();
        let mut command = format!("docker-compose -f {} up -d", compose_file);

        if let Some(service_name) = parameters.get(1) {
            command = format!("{} {}", command, service_name);
        }

        if self.use_sudo {
            command = format!("sudo {}", command);
        }

        command
    }
}
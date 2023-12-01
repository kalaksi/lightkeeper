use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module(
    name="docker-compose-edit",
    version="0.0.1",
    description="Launches an editor for editing a compose-file.",
)]
pub struct Edit {
}

impl Module for Edit {
    fn new(_settings: &HashMap<String, String>) -> Edit {
        Edit {
        }
    }
}

impl CommandModule for Edit {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-compose"),
            parent_id: String::from("docker-compose"),
            multivalue_level: 1,
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("story-editor"),
            display_text: String::from("Edit compose-file"),
            action: UIAction::TextEditor,
            tab_title: String::from("Editor"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, _host: Host, parameters: Vec<String>) -> Result<String, String> {
        let compose_file = parameters.first().unwrap().clone();
        Ok(compose_file)
    }
}
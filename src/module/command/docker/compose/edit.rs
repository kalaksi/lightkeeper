use std::{
    collections::HashMap,
};
use crate::frontend;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("docker-compose-edit", "0.0.1")]
pub struct Edit {
}

impl Module for Edit {
    fn new(_settings: &HashMap<String, String>) -> Self {
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
            ..Default::default()
        }
    }

    fn get_connector_message(&self, parameters: Vec<String>) -> String {
        let compose_file = parameters[0].clone();
        compose_file
    }
}
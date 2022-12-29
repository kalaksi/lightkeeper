use std::{
    collections::HashMap,
    path::Path,
};
use crate::frontend;
use crate::module::{
    Module,
    command::CommandModule,
    command::Command,
    command::UIAction,
    monitoring::docker::compose::ComposeConfig,
    Metadata,
    ModuleSpecification,
};

#[derive(Clone)]
pub struct Edit {
    config: ComposeConfig,
}

impl Module for Edit {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("docker-compose-edit", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(settings: &HashMap<String, String>) -> Self {
        Edit {
            config: ComposeConfig::new(settings),
        }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl CommandModule for Edit {
    fn clone_module(&self) -> Command {
        Box::new(self.clone())
    }

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
        let compose_project_name = parameters.first().unwrap().clone();
        self.config.get_project_compose_file(compose_project_name)
    }
}
use std::collections::HashMap;
use crate::frontend;
use crate::module::connection::ResponseMessage;
use crate::module::{
    Module,
    command::CommandModule,
    command::Command,
    command::CommandResult,
    command::CommandAction,
    Metadata,
    ModuleSpecification,
};

#[derive(Clone)]
pub struct Compose {
    pub main_dir: Option<String>, 
    pub project_directories: Vec<String>, 
}

impl Module for Compose {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("docker-compose", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(settings: &HashMap<String, String>) -> Self {
        // TODO: validation?
        Compose {
            main_dir: settings.get("main_dir").and_then(|value| Some(value.clone())),
            project_directories: settings.get("project_directories").unwrap_or(&String::new()).clone()
                                         .split(",").map(|value| value.to_string()).collect(),
        }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl CommandModule for Compose {
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
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("story-editor"),
            display_text: String::from("Edit compose-file"),
            action: CommandAction::TextEditor,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, parameters: Vec<String>) -> String {
        let result = String::new();

        if let Some(main_dir) = &self.main_dir {

        }

        for project_dir in self.project_directories.iter() {

        }

        format!("")
    }

    fn process_response(&self, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new(response.message.clone()))
    }
}
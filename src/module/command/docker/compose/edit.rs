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
    Metadata,
    ModuleSpecification,
};

#[derive(Clone)]
pub struct Edit {
    pub compose_file_name: String,
    /// If you have one directory under which all the compose projects are, use this.
    pub main_dir: String, 
    /// If you have project directories all over the place, use this.
    pub project_directories: Vec<String>, 
    // TODO
    // pub project_allowlist: Vec<String>, 
    // pub project_blocklist: Vec<String>, 
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
        // TODO: validation?
        Edit {
            compose_file_name: String::from("docker-compose.yml"),
            main_dir: settings.get("main_dir").unwrap_or(&String::new()).clone(),
            project_directories: settings.get("project_directories").unwrap_or(&String::new()).clone()
                                         .split(",").map(|value| value.to_string()).collect(),
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
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("story-editor"),
            display_text: String::from("Edit compose-file"),
            action: UIAction::TextEditor,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, parameters: Vec<String>) -> String {
        let compose_project_name = parameters.first().unwrap().clone();

        let mut project_dir = String::new();

        if !self.main_dir.is_empty() {
            project_dir = Path::new(&self.main_dir).join(compose_project_name.clone()).to_string_lossy().to_string();
        }
        else {
            for dir in self.project_directories.iter() {
                // The last directory component should match the project name.
                let last_dir_component = Path::new(dir).components().last().unwrap().as_os_str().to_string_lossy();
                if compose_project_name == last_dir_component {
                    project_dir = dir.clone();
                }
            }
        }

        let remote_file_path = Path::new(&project_dir)
                                    .join(self.compose_file_name.clone())
                                    .to_string_lossy().to_string();

        remote_file_path
    }
}
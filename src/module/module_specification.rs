use std::fmt::Display;
use strum_macros::{Display, EnumString};

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct ModuleSpecification {
    pub id: String,
    pub version: String,
    pub module_type: ModuleType,
}

impl ModuleSpecification {
    pub fn new(id: &str, version: &str, module_type: ModuleType) -> Self {
        if id.chars().any(char::is_whitespace) {
            panic!("No whitespace allowed in module ID.");
        }

        ModuleSpecification {
            id: id.to_string(),
            version: version.to_string(),
            module_type: module_type,
        }
    }

    pub fn new_with_type(id: &str, version: &str, module_type: ModuleType) -> Self {
        if id.chars().any(char::is_whitespace) {
            panic!("No whitespace allowed in module ID.");
        }

        ModuleSpecification {
            id: id.to_string(),
            version: version.to_string(),
            module_type: module_type,
        }
    }

    pub fn latest_version(&self) -> bool {
        self.version == "latest"
    }
}

impl Display for ModuleSpecification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.id, self.version)
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Display, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ModuleType {
    #[default]
    Unknown,
    Command,
    Monitor,
    Connector,
}
use std::fmt::Display;

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct ModuleSpecification {
    pub id: String,
    pub version: String,
}

impl ModuleSpecification {
    pub fn new(id: &str, version: &str) -> Self {
        if id.chars().any(char::is_whitespace) {
            panic!("No whitespace allowed in module ID.");
        }

        ModuleSpecification {
            id: id.to_string(),
            version: version.to_string(),
        }
    }

    pub fn latest_version(&self) -> bool {
        self.version == "latest"
    }
}

impl Display for ModuleSpecification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.id, self.version)
    }
}
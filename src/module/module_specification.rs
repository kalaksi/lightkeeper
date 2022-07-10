use std::fmt::Display;

// TODO: custom equality comparer for versions
#[derive(Default, Hash, PartialEq, Eq)]
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

    pub fn from_string(string: &String) -> Result<Self, String> {
        let mut parts = string.split('-');
        let id = parts.next().unwrap_or_default();
        let version = parts.next().unwrap_or_default();

        if id.is_empty() || version.is_empty() || id.chars().any(char::is_whitespace)
        {
            return Err(String::from("Invalid specification string"));
        }
        else {
            return Ok(ModuleSpecification::new(id, version))
        }
    }
}

impl Display for ModuleSpecification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.id, self.version)
    }
}

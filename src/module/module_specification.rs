use std::{fmt::Display, str::FromStr};

// TODO: custom equality comparer for versions
#[derive(Default, Hash, PartialEq, Eq)]
pub struct ModuleSpecification {
    pub id: String,
    pub version_spec: String,
}

impl ModuleSpecification {
    pub fn new(id: String, version_spec: String) -> Self {
        ModuleSpecification {
            id: id,
            version_spec: version_spec,
        }
    }

    pub fn from_string(string: &String) -> Result<Self, String> {
        let mut parts = string.split('-');
        let id = parts.next().unwrap_or_default();
        let version = parts.next().unwrap_or_default();

        if (id.is_empty() || version.is_empty())
        {
            return Err(String::from("Invalid specification string"));
        }
        else {
            return Ok(ModuleSpecification::new(String::from(id), String::from(version)))
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }
}

impl Display for ModuleSpecification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.id, self.version_spec)
    }
}
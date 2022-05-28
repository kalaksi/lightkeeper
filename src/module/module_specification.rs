use crate::module::{ Module, connection::ConnectionModule };
use std::fmt::Display;

#[derive(Default)]
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

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_acceptable(&self, module: &Box<dyn ConnectionModule>) -> bool {
        let spec = module.get_module_spec();
        spec.id == self.id && spec.version_spec == self.version_spec
    }
}

impl Display for ModuleSpecification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.id, self.version_spec)
    }
}
use super::ModuleSpecification;

pub struct Metadata {
    pub module_spec: ModuleSpecification,
    pub description: String,
    pub url: String,
    /// Used with extension modules.
    /// Extension modules enrich or modify the original data and are processed after parent module.
    pub parent_module: Option<ModuleSpecification>,
}
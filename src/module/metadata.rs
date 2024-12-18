use std::collections::HashMap;

use super::ModuleSpecification;
use crate::cache::CacheScope;

#[derive(Clone, Debug)]
pub struct Metadata {
    pub module_spec: ModuleSpecification,
    pub description: String,
    pub settings: HashMap<String, String>,
    /// Used with extension modules.
    /// Extension modules enrich or modify the original data and are processed after parent module.
    pub parent_module: Option<ModuleSpecification>,
    /// Stateless modules can be run in parallel. Stateful modules can currently run only 1 connection per host.
    pub is_stateless: bool,
    pub cache_scope: CacheScope,
}

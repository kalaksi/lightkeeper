use std::collections::HashMap;
use crate::module::metadata::Metadata;
use crate::module::ModuleSpecification;

pub trait Module {
    fn new(settings: &HashMap<String, String>) -> Self where Self: Sized;
}

pub trait MetadataSupport {
    fn get_metadata() -> Metadata where Self: Sized;
    fn get_module_spec(&self) -> ModuleSpecification;
}
use std::collections::HashMap;
use super::metadata::Metadata;
use crate::module::ModuleSpecification;

pub trait Module {
    fn new(settings: &HashMap<String, String>) -> Self where Self: Sized;
    fn get_metadata() -> Metadata where Self: Sized;
    fn get_module_spec(&self) -> ModuleSpecification;
}
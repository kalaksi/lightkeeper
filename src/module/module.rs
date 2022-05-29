use super::metadata::Metadata;
use crate::module::ModuleSpecification;

pub trait Module {
    fn new() -> Self where Self: Sized;
    fn get_metadata() -> Metadata where Self: Sized;
    fn get_module_spec(&self) -> ModuleSpecification;

    fn unload(&self) {
    }
}
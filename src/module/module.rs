use super::metadata::Metadata;

pub trait Module {
    fn new() -> Self where Self: Sized;
    fn get_metadata() -> Metadata where Self: Sized;

    fn unload(&self) {
    }
}
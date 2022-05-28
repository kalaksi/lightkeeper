use crate::module::Module;
use crate::module::connection::ConnectionModule;

pub trait CommandModule : Module {
    fn execute(&self, connection: &mut dyn ConnectionModule);

    fn new_command_module() -> Box<dyn CommandModule> where Self: Sized + 'static {
        Box::new(Self::new())
    }
}
extern crate qmetaobject;
use qmetaobject::*;

use crate::command_handler::CommandHandler;

#[derive(QObject, Default)]
pub struct CommandHandlerModel {
    base: qt_base_class!(trait QObject),
    execute: qt_method!(fn(&self, host_id: QString, command_id: QString)),
    command_handler: CommandHandler,
}

impl CommandHandlerModel {
    pub fn new(command_handler: CommandHandler) -> Self {
        CommandHandlerModel { 
            command_handler: command_handler,
            ..Default::default()
        }
    }

    fn execute(&mut self, host_id: QString, command_id: QString) {
        self.command_handler.execute(host_id.to_string(), command_id.to_string())
    }
}
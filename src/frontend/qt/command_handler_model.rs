extern crate qmetaobject;
use qmetaobject::*;

use crate::command_handler::CommandHandler;

#[derive(QObject, Default)]
pub struct CommandHandlerModel {
    base: qt_base_class!(trait QObject),
    get_commands: qt_method!(fn(&self, host_id: QString) -> QVariantList),
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

    fn get_commands(&self, host_id: QString) -> QVariantList {
        let commands_with_parameters = self.command_handler.get_host_commands(host_id.to_string());

        let mut full_commands = QVariantList::default();
        for (command_id, parameters) in commands_with_parameters.iter() {
            for parameter in parameters.iter() {
                full_commands.push(format!("{} {}", command_id, parameter).to_qvariant());
            }
        }

        full_commands
    }

    fn execute(&mut self, host_id: QString, command_id: QString) {
        self.command_handler.execute(host_id.to_string(), command_id.to_string())
    }
}
extern crate qmetaobject;
use qmetaobject::*;

use crate::command_handler::CommandHandler;

#[derive(QObject, Default)]
pub struct CommandHandlerModel {
    base: qt_base_class!(trait QObject),
    get_commands: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_child_commands: qt_method!(fn(&self, host_id: QString, parent_id: QString) -> QVariantList),
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
        let command_datas = self.command_handler.get_host_commands(host_id.to_string());

        command_datas.values().map(|item| serde_json::to_string(&item).unwrap().to_qvariant())
                              .collect()
    }

    fn get_child_commands(&self, host_id: QString, parent_id: QString) -> QVariantList {
        let command_datas = self.command_handler.get_host_commands(host_id.to_string());

        command_datas.values().filter(|item| item.display_options.parent_id == parent_id.to_string())
                              .map(|item| serde_json::to_string(&item).unwrap().to_qvariant())
                              .collect()
    }

    fn execute(&mut self, host_id: QString, command_id: QString) {
        self.command_handler.execute(host_id.to_string(), command_id.to_string())
    }
}
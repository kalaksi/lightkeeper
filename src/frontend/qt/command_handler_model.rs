extern crate qmetaobject;
use qmetaobject::*;

use crate::command_handler::{CommandHandler, CommandData};
use crate::module::command::CommandAction;

#[derive(QObject, Default)]
pub struct CommandHandlerModel {
    base: qt_base_class!(trait QObject),
    get_commands: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_child_commands: qt_method!(fn(&self, host_id: QString, parent_id: QString) -> QVariantList),
    execute: qt_method!(fn(&self, host_id: QString, command_id: QString, target_id: QString)),

    dialog_opened: qt_signal!(),
    confirmation_dialog_opened: qt_signal!(),

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
        let all_commands = self.command_handler.get_host_commands(host_id.to_string());
        let mut valid_commands = all_commands.values().filter(|item| item.display_options.parent_id == parent_id.to_string())
                                                      .collect::<Vec<&CommandData>>();

        valid_commands.sort_by(|left, right| left.display_options.display_priority.cmp(&right.display_options.display_priority));
        valid_commands.iter().map(|item| serde_json::to_string(&item).unwrap().to_qvariant()).collect()
    }

    fn execute(&mut self, host_id: QString, command_id: QString, target_id: QString) {
        let display_options = self.command_handler.get_host_command(host_id.to_string(), command_id.to_string()).display_options;

        if !display_options.confirmation_text.is_empty() {
            self.confirmation_dialog_opened();
            return;
        }

        let action = self.command_handler.execute(host_id.to_string(), command_id.to_string(), target_id.to_string());

        // The UI action to be triggered after successful execution.
        match action {
            CommandAction::None => {},
            CommandAction::Dialog => self.dialog_opened(),
        }
    }
}
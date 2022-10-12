extern crate qmetaobject;
use qmetaobject::*;

use crate::command_handler::{CommandHandler, CommandData};
use crate::module::command::CommandAction;

#[derive(QObject, Default)]
pub struct CommandHandlerModel {
    base: qt_base_class!(trait QObject),
    get_commands: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_child_commands: qt_method!(fn(&self, host_id: QString, parent_id: QString) -> QVariantList),
    execute: qt_method!(fn(&self, host_id: QString, command_id: QString, target_id: QString) -> u64),
    execute_confirmed: qt_method!(fn(&self, host_id: QString, command_id: QString, target_id: QString) -> u64),

    // Signal to open a dialog. Since execution is async, invocation_id is used to retrieve the matching result.
    text_dialog_opened: qt_signal!(invocation_id: u64),
    text_view_opened: qt_signal!(headerText: QString, invocation_id: u64),
    confirmation_dialog_opened: qt_signal!(text: QString, host_id: QString, command_id: QString, target_id: QString),

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

    fn execute(&mut self, host_id: QString, command_id: QString, target_id: QString) -> u64 {
        let display_options = self.command_handler.get_host_command(host_id.to_string(), command_id.to_string()).display_options;

        if display_options.confirmation_text.is_empty() {
            return self.execute_confirmed(host_id, command_id, target_id);
        }
        else {
            self.confirmation_dialog_opened(QString::from(display_options.confirmation_text), host_id, command_id, target_id);
        }

        return 0
    }

    fn execute_confirmed(&mut self, host_id: QString, command_id: QString, target_id: QString) -> u64 {
        let invocation_id = 0;

        // The UI action to be triggered after successful execution.
        let display_options = self.command_handler.get_host_command(host_id.to_string(), command_id.to_string()).display_options;
        match display_options.action {
            CommandAction::None => {
                self.command_handler.execute(host_id.to_string(), command_id.to_string(), target_id.to_string());
            },
            CommandAction::Dialog => {
                self.command_handler.execute(host_id.to_string(), command_id.to_string(), target_id.to_string());
                self.text_dialog_opened(invocation_id)
            },
            CommandAction::TextView => {
                self.command_handler.execute(host_id.to_string(), command_id.to_string(), target_id.to_string());
                self.text_view_opened(QString::from(format!("{}: {}", command_id.to_string(), target_id.to_string())), invocation_id)
            },
            CommandAction::Terminal => {
                self.command_handler.open_terminal(vec![
                    String::from("ssh"),
                    String::from("-t"),
                    host_id.to_string(),
                    // TODO: allow only alphanumeric and dashes (no spaces and no leading dash)
                    format!("sudo docker exec -it {} /bin/sh", target_id.to_string())]
                )
            }
        }

        return invocation_id
    }
}
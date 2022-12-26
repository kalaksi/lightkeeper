extern crate qmetaobject;
use std::collections::HashMap;

use qmetaobject::*;

use crate::command_handler::{CommandHandler, CommandData};
use crate::module::command::UIAction;

#[derive(QObject, Default)]
pub struct CommandHandlerModel {
    base: qt_base_class!(trait QObject),
    get_commands: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_child_commands: qt_method!(fn(&self, host_id: QString, category: QString, parent_id: QString) -> QVariantList),
    execute: qt_method!(fn(&self, host_id: QString, command_id: QString, parameters: QVariantList) -> u64),
    execute_confirmed: qt_method!(fn(&self, host_id: QString, command_id: QString, parameters: QVariantList) -> u64),

    // Signal to open a dialog. Since execution is async, invocation_id is used to retrieve the matching result.
    details_dialog_opened: qt_signal!(invocation_id: u64),
    details_subview_opened: qt_signal!(headerText: QString, invocation_id: u64),
    // TODO: dialog for logs (refactor so doesn't need dedicated)
    logs_subview_opened: qt_signal!(headerText: QString, invocation_id: u64),
    text_editor_opened: qt_signal!(headerText: QString, invocation_id: u64),
    confirmation_dialog_opened: qt_signal!(text: QString, host_id: QString, command_id: QString, parameters: QVariantList),

    command_handler: CommandHandler,
    command_display_order: Vec<String>,
}

impl CommandHandlerModel {
    pub fn new(command_handler: CommandHandler, command_display_order: Vec<String>) -> Self {
        CommandHandlerModel { 
            command_handler: command_handler,
            command_display_order: command_display_order,
            ..Default::default()
        }
    }

    fn get_commands(&self, host_id: QString) -> QVariantList {
        let command_datas = self.command_handler.get_host_commands(host_id.to_string());

        command_datas.values().map(|item| serde_json::to_string(&item).unwrap().to_qvariant()).collect()
    }


    // Parent ID is either command ID or category ID (for category-level commands).
    fn get_child_commands(&self, host_id: QString, category: QString, parent_id: QString) -> QVariantList {
        let category_string = category.to_string().to_lowercase();
        let parent_id_string = parent_id.to_string().to_lowercase();
        let mut all_commands = self.command_handler.get_host_commands(host_id.to_string())
                                   .into_iter().filter(|(_, data)| data.display_options.parent_id == parent_id_string &&
                                                                   data.display_options.category == category_string)
                                   .collect::<HashMap<String, CommandData>>();

        let mut valid_commands_sorted = Vec::<CommandData>::new();
        for command_id in self.command_display_order.iter() {
            if let Some(command_data) = all_commands.remove(command_id) {
                valid_commands_sorted.push(command_data);
            }
        }

        // Append the rest of the commands in an undefined order.
        let mut unsorted_commands = all_commands.into_iter().map(|(_, command)| command).collect();
        valid_commands_sorted.append(&mut unsorted_commands);

        // Return list of JSONs.
        valid_commands_sorted.iter().map(|item| serde_json::to_string(&item).unwrap().to_qvariant()).collect()
    }

    fn execute(&mut self, host_id: QString, command_id: QString, parameters: QVariantList) -> u64 {
        let display_options = self.command_handler.get_host_command(host_id.to_string(), command_id.to_string()).display_options;

        if display_options.confirmation_text.is_empty() {
            return self.execute_confirmed(host_id, command_id, parameters);
        }
        else {
            self.confirmation_dialog_opened(QString::from(display_options.confirmation_text), host_id, command_id, parameters);
        }

        return 0
    }

    fn execute_confirmed(&mut self, host_id: QString, command_id: QString, parameters: QVariantList) -> u64 {
        let mut invocation_id = 0;
        let parameters: Vec<String> = parameters.into_iter().map(|qvar| qvar.to_qbytearray().to_string()).collect();

        let display_options = self.command_handler.get_host_command(host_id.to_string(), command_id.to_string()).display_options;
        match display_options.action {
            UIAction::None => {
                invocation_id = self.command_handler.execute(host_id.to_string(), command_id.to_string(), parameters);
            },
            UIAction::Dialog => {
                invocation_id = self.command_handler.execute(host_id.to_string(), command_id.to_string(), parameters);
                self.details_dialog_opened(invocation_id)
            },
            UIAction::TextView => {
                let target_id = parameters.first().unwrap().clone();
                invocation_id = self.command_handler.execute(host_id.to_string(), command_id.to_string(), parameters);
                self.details_subview_opened(QString::from(format!("{}: {}", command_id.to_string(), target_id)), invocation_id)
            },
            UIAction::LogView => {
                let target_id = parameters.first().unwrap().clone();
                invocation_id = self.command_handler.execute(host_id.to_string(), command_id.to_string(), parameters);
                self.logs_subview_opened(QString::from(format!("{}: {}", command_id.to_string(), target_id)), invocation_id)
            },
            UIAction::Terminal => {
                let target_id = parameters.first().unwrap();
                // TODO: don't hardcode the command here (add to command module).
                self.command_handler.open_terminal(vec![
                    String::from("ssh"),
                    String::from("-t"),
                    host_id.to_string(),
                    // TODO: allow only alphanumeric and dashes (no spaces and no leading dash).
                    format!("sudo docker exec -it {} /bin/sh", target_id.to_string())
                ])
            },
            UIAction::TextEditor => {
                // TODO: integrated text editor
                let remote_file_path = parameters.first().unwrap().clone();
                self.command_handler.open_text_editor(host_id.to_string(), command_id.to_string(), remote_file_path);
            },
        }

        return invocation_id
    }
}
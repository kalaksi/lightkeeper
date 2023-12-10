extern crate qmetaobject;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use qmetaobject::*;

use crate::command_handler::{CommandHandler, CommandData};
use crate::configuration;
use crate::connection_manager::{CachePolicy, ConnectionManager};
use crate::host_manager;
use crate::module::command::UIAction;
use crate::monitor_manager::MonitorManager;

// This should probably be renamed to something like RequestHandlerModel.
#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct CommandHandlerModel {
    base: qt_base_class!(trait QObject),

    stop: qt_method!(fn(&mut self)),
    reconfigure: qt_method!(fn(&mut self, config: QVariant, hosts_config: QVariant)),

    get_all_host_categories: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_category_commands: qt_method!(fn(&self, host_id: QString, category: QString) -> QVariantList),
    get_commands_on_level: qt_method!(fn(&self, host_id: QString, category: QString, parent_id: QString, multivalue_level: QString) -> QVariantList),
    get_child_command_count: qt_method!(fn(&self, host_id: QString, category: QString) -> u32),
    execute: qt_method!(fn(&self, host_id: QString, command_id: QString, parameters: QStringList)),
    execute_confirmed: qt_method!(fn(&self, host_id: QString, command_id: QString, parameters: QStringList)),
    executePlain: qt_method!(fn(&self, host_id: QString, command_id: QString, parameters: QStringList) -> u64),
    saveAndUploadFile: qt_method!(fn(&self, host_id: QString, command_id: QString, local_file_path: QString, contents: QString) -> u64),
    removeFile: qt_method!(fn(&self, local_file_path: QString)),
    hasFileChanged: qt_method!(fn(&self, local_file_path: QString, contents: QString) -> bool),

    // Host initialization methods.
    initialize_host: qt_method!(fn(&self, host_id: QString)),
    force_initialize_host: qt_method!(fn(&self, host_id: QString)),
    force_initialize_hosts: qt_method!(fn(&self)),

    // Monitor refresh methods.
    force_refresh_monitors_of_command: qt_method!(fn(&self, host_id: QString, command_id: QString) -> QVariantList),
    cached_refresh_monitors_of_category: qt_method!(fn(&self, host_id: QString, category: QString) -> QVariantList),
    refresh_monitors_of_category: qt_method!(fn(&self, host_id: QString, category: QString) -> QVariantList),
    force_refresh_monitors_of_category: qt_method!(fn(&self, host_id: QString, category: QString) -> QVariantList),

    // Signal to open a dialog. Since execution is async, invocation_id is used to retrieve the matching result.
    details_dialog_opened: qt_signal!(invocation_id: u64),
    input_dialog_opened: qt_signal!(input_specs: QString, host_id: QString, command_id: QString, parameters: QStringList),
    text_dialog_opened: qt_signal!(invocation_id: u64),
    confirmation_dialog_opened: qt_signal!(text: QString, host_id: QString, command_id: QString, parameters: QStringList),
    textViewOpened: qt_signal!(title: QString, invocation_id: u64),
    textEditorViewOpened: qt_signal!(header_text: QString, invocation_id: u64, local_file_path: QString),
    terminalViewOpened: qt_signal!(header_text: QString, command: QStringList),
    logsViewOpened: qt_signal!(time_controls: bool, title: QString, command_id: QString, parameters: QStringList, invocation_id: u64),
    command_executed: qt_signal!(invocation_id: u64, host_id: QString, command_id: QString, category: QString, button_identifier: QString),
    // Platform info refresh was just triggered.
    host_initializing: qt_signal!(host_id: QString),

    command_handler: CommandHandler,
    monitor_manager: MonitorManager,
    configuration: configuration::Configuration,

    // These are here only so that reconfiguring and stopping can be controlled by GUI.
    connection_manager: ConnectionManager,
    host_manager: Rc<RefCell<host_manager::HostManager>>,
}

#[allow(non_snake_case)]
impl CommandHandlerModel {
    pub fn new(
        command_handler: CommandHandler,
        monitor_manager: MonitorManager,
        connection_manager: ConnectionManager,
        host_manager: Rc<RefCell<host_manager::HostManager>>,
        configuration: configuration::Configuration) -> Self {
        CommandHandlerModel { 
            command_handler: command_handler,
            monitor_manager: monitor_manager,
            connection_manager: connection_manager,
            host_manager: host_manager,
            configuration: configuration,
            ..Default::default()
        }
    }

    fn stop(&mut self) {
        self.connection_manager.stop();
        self.host_manager.borrow_mut().exit();
    }

    fn reconfigure(&mut self, main_config: QVariant, hosts_config: QVariant) {
        let main_config = configuration::Configuration::from_qvariant(main_config).unwrap();
        let hosts_config = configuration::Hosts::from_qvariant(hosts_config).unwrap();
        self.configuration = main_config.clone();

        self.host_manager.borrow_mut().configure(&hosts_config);
        self.host_manager.borrow_mut().start_receiving_updates();

        self.connection_manager.configure(&hosts_config, &main_config.cache_settings);
        self.connection_manager.start_processing_requests();

        self.monitor_manager.configure(
            &hosts_config,
            self.connection_manager.new_request_sender(),
            self.host_manager.borrow().new_state_update_sender()
        );
        self.command_handler.configure(
            &hosts_config,
            &main_config.preferences,
            self.connection_manager.new_request_sender(),
            self.host_manager.borrow().new_state_update_sender()
        );
    }

    // Return CommandDatas relevant to category as QVariants.
    fn get_category_commands(&self, host_id: QString, category: QString) -> QVariantList {
        let category_string = category.to_string();

        let mut category_commands = self.command_handler.get_commands_for_host(host_id.to_string())
                                                        .into_iter().filter(|(_, data)| data.display_options.category == category_string)
                                                        .map(|(_, data)| data)
                                                        .collect::<Vec<CommandData>>();

        let command_order = match self.configuration.display_options.as_ref().unwrap().categories.get(&category_string) {
            Some(category_data) => category_data.command_order.clone().unwrap_or_default(),
            None => Vec::new(),
        };

        // Orders first by predefined order and then alphabetically.
        category_commands.sort_by_key(|command_data| {
            // Priority will be the position in the predefined order or (shared) last priority if not found.
            let priority = command_order.iter().position(|id| id == &command_data.command_id)
                                               .unwrap_or(command_order.len());

            // Tuple for sorting by priority and then by id.
            (priority, command_data.command_id.clone())
        });

        let mut result = QVariantList::default();
        for command_data in category_commands {
            result.push(command_data.to_qvariant());
        }
        result
    }

    // `parent_id` is either command ID or category ID (for category-level commands).
    // Returns CommandData as JSON strings.
    fn get_commands_on_level(&self, host_id: QString, category: QString, parent_id: QString, multivalue_level: QString) -> QVariantList {
        let category_string = category.to_string();
        let parent_id_string = parent_id.to_string();
        let multivalue_level: u8 = multivalue_level.to_string().parse().unwrap();

        let mut all_commands = self.command_handler.get_commands_for_host(host_id.to_string())
                                   .into_iter().filter(|(_, data)| 
                                        data.display_options.parent_id == parent_id_string &&
                                        data.display_options.category == category_string &&
                                        (data.display_options.multivalue_level == 0 || data.display_options.multivalue_level == multivalue_level))
                                   .collect::<HashMap<String, CommandData>>();

        let mut valid_commands_sorted = Vec::<CommandData>::new();

        let command_order = match self.configuration.display_options.as_ref().unwrap().categories.get(&category_string) {
            Some(category_data) => category_data.command_order.clone().unwrap_or_default(),
            None => Vec::new(),
        };

        for command_id in command_order.iter() {
            if let Some(command_data) = all_commands.remove(command_id) {
                valid_commands_sorted.push(command_data);
            }
        }

        // Append the rest of the commands in alphabetical order.
        let mut rest_of_commands: Vec<CommandData> = all_commands.into_values().collect();
        rest_of_commands.sort_by(|left, right| left.command_id.cmp(&right.command_id));
        valid_commands_sorted.append(&mut rest_of_commands);

        // Return list of JSONs.
        valid_commands_sorted.iter().map(|item| serde_json::to_string(&item).unwrap().to_qvariant()).collect()
    }

    fn get_child_command_count(&self, host_id: QString, category: QString) -> u32 {
        let category_string = category.to_string();

        self.command_handler.get_commands_for_host(host_id.to_string()).into_iter()
                            .filter(|(_, data)| data.display_options.category == category_string &&
                                                !data.display_options.parent_id.is_empty())
                            .count() as u32
    }

    fn execute(&mut self, host_id: QString, command_id: QString, parameters: QStringList) {
        let display_options = self.command_handler.get_command_for_host(&host_id.to_string(), &command_id.to_string()).display_options;

        if !display_options.user_parameters.is_empty() {
            let input_specs: QString = QString::from(serde_json::to_string(&display_options.user_parameters).unwrap());
            self.input_dialog_opened(input_specs, host_id, command_id, parameters);
        }
        else if !display_options.confirmation_text.is_empty() {
            self.confirmation_dialog_opened(QString::from(display_options.confirmation_text), host_id, command_id, parameters);
        }
        else {
            self.execute_confirmed(host_id, command_id, parameters);
        }
    }

    fn execute_confirmed(&mut self, host_id: QString, command_id: QString, parameters: QStringList) {
        let host_id = host_id.to_string();
        let command_id = command_id.to_string();
        let parameters: Vec<String> = parameters.into_iter().map(|qvar| qvar.to_string()).collect();

        let display_options = self.command_handler.get_command_for_host(&host_id, &command_id).display_options;
        match display_options.action {
            UIAction::None => {
                let invocation_id = self.command_handler.execute(&host_id, &command_id, &parameters);

                if invocation_id > 0 {
                    let button_identifier = format!("{}|{}", command_id, parameters.first().unwrap_or(&String::new()));
                    self.command_executed(invocation_id, host_id.into(), command_id.into(), display_options.category.into(), button_identifier.into());
                }
            },
            UIAction::DetailsDialog => {
                let invocation_id = self.command_handler.execute(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    self.details_dialog_opened(invocation_id)
                }
            },
            UIAction::TextView => {
                let target_id = parameters.first().unwrap().clone();
                let invocation_id = self.command_handler.execute(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    self.textViewOpened(QString::from(format!("{}: {}", command_id, target_id)), invocation_id)
                }
            },
            UIAction::TextDialog => {
                let invocation_id = self.command_handler.execute(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    self.text_dialog_opened(invocation_id)
                }
            },
            UIAction::LogView => {
                let invocation_id = self.command_handler.execute(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    let parameters_qs = parameters.into_iter().map(QString::from).collect::<QStringList>();
                    self.logsViewOpened(false, QString::from(display_options.tab_title), QString::from(command_id), parameters_qs, invocation_id);
                }
            },
            UIAction::LogViewWithTimeControls => {
                let invocation_id = self.command_handler.execute(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    let parameters_qs = parameters.into_iter().map(QString::from).collect::<QStringList>();
                    self.logsViewOpened(true, QString::from(display_options.tab_title), QString::from(command_id), parameters_qs, invocation_id);
                }
            },
            UIAction::Terminal => {
                if self.configuration.preferences.terminal == configuration::INTERNAL {
                    let command = self.command_handler.open_remote_terminal_command(&host_id, &command_id, &parameters);
                    let command_qsl = command.to_vec().into_iter().map(QString::from).collect::<QStringList>();
                    self.terminalViewOpened(QString::from(display_options.tab_title), command_qsl)
                }
                else {
                    self.command_handler.open_external_terminal(&host_id, &command_id, parameters);
                }
            },
            UIAction::TextEditor => {
                let remote_file_path = parameters.first().unwrap().clone();
                if self.configuration.preferences.use_remote_editor {
                    if self.configuration.preferences.terminal == configuration::INTERNAL {
                        let command = self.command_handler.open_remote_text_editor_command(&host_id, &remote_file_path);
                        let command_qsl = command.to_vec().into_iter().map(QString::from).collect::<QStringList>();
                        self.terminalViewOpened(QString::from(display_options.tab_title), command_qsl);
                    }
                    else {
                        self.command_handler.open_external_terminal(&host_id, &command_id, parameters);
                    }
                }
                else {
                    if self.configuration.preferences.text_editor == configuration::INTERNAL {
                        let (invocation_id, file_contents) = self.command_handler.download_editable_file(&host_id, &command_id, &remote_file_path); 
                        self.textEditorViewOpened(QString::from(command_id), invocation_id, QString::from(file_contents));
                    }
                    else {
                        self.command_handler.open_external_text_editor(&host_id, &command_id, &remote_file_path);
                    }
                }
            },
        }
    }

    fn executePlain(&mut self, host_id: QString, command_id: QString, parameters: QStringList) -> u64 {
        let host_id = host_id.to_string();
        let command_id = command_id.to_string();
        let parameters: Vec<String> = parameters.into_iter().map(|qvar| qvar.to_string()).collect();
        self.command_handler.execute(&host_id, &command_id, &parameters)
    }

    fn saveAndUploadFile(&mut self, host_id: QString, command_id: QString, local_file_path: QString, contents: QString) -> u64 {
        let host_id = host_id.to_string();
        let command_id = command_id.to_string();
        let local_file_path = local_file_path.to_string();
        let contents = contents.to_string().into_bytes();

        self.command_handler.write_file(&local_file_path, contents);
        let invocation_id = self.command_handler.upload_file(&host_id, &command_id, &local_file_path);
        invocation_id
    }

    fn removeFile(&mut self, local_file_path: QString) {
        let local_file_path = local_file_path.to_string();
        self.command_handler.remove_file(&local_file_path);
    }

    fn hasFileChanged(&self, local_file_path: QString, contents: QString) -> bool {
        let local_file_path = local_file_path.to_string();
        let contents = contents.to_string().into_bytes();
        self.command_handler.has_file_changed(&local_file_path, contents)
    }

    fn initialize_host(&mut self, host_id: QString) {
        self.monitor_manager.refresh_platform_info(&host_id.to_string(), None);
        self.host_initializing(host_id);
    }

    fn force_initialize_host(&mut self, host_id: QString) {
        self.monitor_manager.refresh_platform_info(&host_id.to_string(), Some(CachePolicy::BypassCache));
        self.host_initializing(host_id);
    }

    fn force_initialize_hosts(&mut self) {
        let host_ids = self.monitor_manager.refresh_platform_info_all(Some(CachePolicy::BypassCache));
        for host_id in host_ids {
            self.host_initializing(QString::from(host_id));
        }
    }

    // Finds related monitors for a command and refresh them.
    fn force_refresh_monitors_of_command(&mut self, host_id: QString, command_id: QString) -> QVariantList  {
        let host_id = host_id.to_string();
        let command_id = command_id.to_string();

        if host_id.is_empty() || command_id.is_empty() {
            ::log::error!("Invalid parameters: {}, {}", host_id, command_id);
            return QVariantList::default();
        }

        ::log::debug!("[{}] Refreshing monitors related to command {}", host_id, command_id);

        let command = self.command_handler.get_command_for_host(&host_id, &command_id);
        let monitor_id = command.display_options.parent_id;

        let invocation_ids = self.monitor_manager.refresh_monitors_by_id(&host_id, &monitor_id, CachePolicy::BypassCache);
        QVariantList::from_iter(invocation_ids)
    }

    fn cached_refresh_monitors_of_category(&mut self, host_id: QString, category: QString) -> QVariantList {
        let invocation_ids = self.monitor_manager.refresh_monitors_of_category_control(&host_id.to_string(), &category.to_string(), CachePolicy::OnlyCache);
        QVariantList::from_iter(invocation_ids)
    }

    fn refresh_monitors_of_category(&mut self, host_id: QString, category: QString) -> QVariantList {
        let invocation_ids = self.monitor_manager.refresh_monitors_of_category(&host_id.to_string(), &category.to_string());
        QVariantList::from_iter(invocation_ids)
    }

    fn force_refresh_monitors_of_category(&mut self, host_id: QString, category: QString) -> QVariantList {
        let invocation_ids = self.monitor_manager.refresh_monitors_of_category_control(&host_id.to_string(), &category.to_string(), CachePolicy::BypassCache);
        QVariantList::from_iter(invocation_ids)
    }

    fn get_all_host_categories(&self, host_id: QString) -> QVariantList {
        if host_id.is_empty() {
            return QVariantList::default()
        }

        self.monitor_manager.get_all_host_categories(&host_id.to_string())
                            .iter().map(|category| category.to_qvariant()).collect()
    }
}
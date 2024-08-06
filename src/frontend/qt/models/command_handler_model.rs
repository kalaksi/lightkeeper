extern crate qmetaobject;
use std::collections::HashMap;
use std::sync::mpsc;

use qmetaobject::*;

use crate::command_handler::{CommandHandler, CommandData};
use crate::configuration;
use crate::connection_manager::{CachePolicy, ConnectorRequest};
use crate::host_manager::StateUpdateMessage;
use crate::module::command::UIAction;
use crate::monitor_manager::MonitorManager;


// This should probably be renamed to something like RequestHandlerModel.
#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct CommandHandlerModel {
    base: qt_base_class!(trait QObject),

    //
    // Slots
    //
    getAllHostCategories: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    getCategoryCommands: qt_method!(fn(&self, host_id: QString, category: QString) -> QVariantList),
    getCommandsOnLevel: qt_method!(fn(&self, host_id: QString, category: QString, parent_id: QString, multivalue_level: QString) -> QVariantList),
    get_child_command_count: qt_method!(fn(&self, host_id: QString, category: QString) -> u32),
    execute: qt_method!(fn(&self, button_id: QString, host_id: QString, command_id: QString, parameters: QStringList)),
    executeConfirmed: qt_method!(fn(&self, button_id: QString, host_id: QString, command_id: QString, parameters: QStringList)),
    executePlain: qt_method!(fn(&self, host_id: QString, command_id: QString, parameters: QStringList) -> u64),
    saveAndUploadFile: qt_method!(fn(&self, host_id: QString, command_id: QString, local_file_path: QString, contents: QString) -> u64),
    removeFile: qt_method!(fn(&self, local_file_path: QString)),
    hasFileChanged: qt_method!(fn(&self, local_file_path: QString, contents: QString) -> bool),
    verifyHostKey: qt_method!(fn(&self, host_id: QString, connector_id: QString, key_id: QString)),

    // Host initialization methods.
    initializeHost: qt_method!(fn(&self, host_id: QString)),
    forceInitializeHost: qt_method!(fn(&self, host_id: QString)),
    forceInitializeHosts: qt_method!(fn(&self)),

    // Monitor refresh methods.
    forceRefreshMonitorsOfCommand: qt_method!(fn(&self, host_id: QString, command_id: QString) -> QVariantList),
    forceRefreshMonitorsOfCategory: qt_method!(fn(&self, host_id: QString, category: QString) -> QVariantList),

    //
    // Signals
    //

    // Signal to open a dialog. Since execution is async, invocation_id is used to retrieve the matching result.
    detailsDialogOpened: qt_signal!(invocation_id: u64),
    inputDialogOpened: qt_signal!(input_specs: QString, button_id: QString, host_id: QString, command_id: QString, parameters: QStringList),
    textDialogOpened: qt_signal!(invocation_id: u64),
    confirmationDialogOpened: qt_signal!(text: QString, button_id: QString, host_id: QString, command_id: QString, parameters: QStringList),
    commandOutputDialogOpened: qt_signal!(title: QString, invocation_id: u64),
    textViewOpened: qt_signal!(title: QString, invocation_id: u64),
    textEditorViewOpened: qt_signal!(header_text: QString, invocation_id: u64, local_file_path: QString),
    terminalViewOpened: qt_signal!(header_text: QString, command: QStringList),
    logsViewOpened: qt_signal!(time_controls: bool, title: QString, command_id: QString, parameters: QStringList, invocation_id: u64),
    commandExecuted: qt_signal!(invocation_id: u64, host_id: QString, command_id: QString, category: QString, button_identifier: QString),
    // Platform info refresh was just triggered.
    hostInitializing: qt_signal!(host_id: QString),

    //
    // Private properties
    //
    command_handler: CommandHandler,
    monitor_manager: MonitorManager,
    configuration: configuration::Configuration,
}

#[allow(non_snake_case)]
impl CommandHandlerModel {
    pub fn new(
        command_handler: CommandHandler,
        monitor_manager: MonitorManager,
        configuration: configuration::Configuration) -> Self {

        CommandHandlerModel { 
            command_handler: command_handler,
            monitor_manager: monitor_manager,
            configuration: configuration,
            ..Default::default()
        }
    }

    pub fn configure(&mut self,
        main_config: &configuration::Configuration,
        hosts_config: &configuration::Hosts,
        request_sender: mpsc::Sender<ConnectorRequest>,
        update_sender: mpsc::Sender<StateUpdateMessage>
    ) {
        self.configuration = main_config.clone();
        self.monitor_manager.configure(&hosts_config, request_sender.clone(), update_sender.clone());
        self.command_handler.configure(&hosts_config, &main_config.preferences, request_sender, update_sender);
    }

    pub fn start_processing_responses(&mut self) {
        self.monitor_manager.start_processing_responses();
        self.command_handler.start_processing_responses();
    }

    pub fn stop(&mut self) {
        self.command_handler.stop();
        self.monitor_manager.stop();
    }

    pub fn refresh_host_monitors(&mut self, host_id: String, cache_policy: Option<CachePolicy>) {
        for category in self.monitor_manager.get_all_host_categories(&host_id) {
            let _invocation_ids = self.monitor_manager.refresh_monitors_of_category(&host_id, &category, cache_policy);
        }
    }

    // Return CommandDatas relevant to category as QVariants.
    fn getCategoryCommands(&self, host_id: QString, category: QString) -> QVariantList {
        let category_string = category.to_string();

        let mut category_commands = self.command_handler.get_commands_for_host(host_id.to_string())
                                                        .into_iter().filter(|(_, data)| data.display_options.category == category_string)
                                                        .map(|(_, data)| data)
                                                        .collect::<Vec<CommandData>>();

        let command_order = match self.configuration.display_options.categories.get(&category_string) {
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
    fn getCommandsOnLevel(&self, host_id: QString, category: QString, parent_id: QString, multivalue_level: QString) -> QVariantList {
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

        let command_order = match self.configuration.display_options.categories.get(&category_string) {
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

    fn execute(&mut self, button_id: QString, host_id: QString, command_id: QString, parameters: QStringList) {
        let display_options = match self.command_handler.get_command_for_host(&host_id.to_string(), &command_id.to_string()) {
            Some(command_data) => command_data.display_options,
            None => return,
        };

        if !display_options.user_parameters.is_empty() {
            let input_specs: QString = QString::from(serde_json::to_string(&display_options.user_parameters).unwrap());
            self.inputDialogOpened(input_specs, button_id, host_id, command_id, parameters);
        }
        else if !display_options.confirmation_text.is_empty() {
            self.confirmationDialogOpened(QString::from(display_options.confirmation_text), button_id, host_id, command_id, parameters);
        }
        else {
            self.executeConfirmed(button_id, host_id, command_id, parameters);
        }
    }

    fn executeConfirmed(&mut self, button_id: QString, host_id: QString, command_id: QString, parameters: QStringList) {
        let host_id = host_id.to_string();
        let command_id = command_id.to_string();
        let parameters: Vec<String> = parameters.into_iter().map(|qvar| qvar.to_string()).collect();

        let display_options = match self.command_handler.get_command_for_host(&host_id, &command_id) {
            Some(command_data) => command_data.display_options,
            None => return,
        };

        match display_options.action {
            UIAction::None => {
                let invocation_id = self.command_handler.execute(&host_id, &command_id, &parameters);

                if invocation_id > 0 {
                    self.commandExecuted(invocation_id, host_id.into(), command_id.into(), display_options.category.into(), button_id.into());
                }
            },
            UIAction::FollowOutput => {
                let invocation_id = self.command_handler.execute(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    let title = QString::from(format!("{}: {}", command_id, parameters.first().unwrap_or(&String::new())));
                    self.commandOutputDialogOpened(title, invocation_id);
                    self.commandExecuted(invocation_id, host_id.into(), command_id.into(), display_options.category.into(), button_id.into());
                }
            },
            UIAction::DetailsDialog => {
                let invocation_id = self.command_handler.execute(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    self.detailsDialogOpened(invocation_id)
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
                    self.textDialogOpened(invocation_id)
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
                        let command = self.command_handler.open_remote_text_editor(&host_id, &remote_file_path);
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
                        let local_file_path = self.command_handler.open_external_text_editor(&host_id, &command_id, &remote_file_path);
                        let _invocation_id = self.command_handler.upload_file(&host_id, &command_id, &local_file_path);
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

    fn verifyHostKey(&self, host_id: QString, connector_id: QString, key_id: QString) {
        let host_id = host_id.to_string();
        let connector_id = connector_id.to_string();
        let key_id = key_id.to_string();
        self.command_handler.verify_host_key(&host_id, &connector_id, &key_id);
    }

    fn initializeHost(&mut self, host_id: QString) {
        self.monitor_manager.refresh_platform_info(&host_id.to_string(), None);
        self.hostInitializing(host_id);
    }

    fn forceInitializeHost(&mut self, host_id: QString) {
        self.monitor_manager.refresh_platform_info(&host_id.to_string(), Some(CachePolicy::BypassCache));
        self.hostInitializing(host_id);
    }

    fn forceInitializeHosts(&mut self) {
        let host_ids = self.monitor_manager.refresh_platform_info_all(Some(CachePolicy::BypassCache));
        for host_id in host_ids {
            self.hostInitializing(QString::from(host_id));
        }
    }

    // Finds related monitors for a command and refresh them.
    fn forceRefreshMonitorsOfCommand(&mut self, host_id: QString, command_id: QString) -> QVariantList  {
        let host_id = host_id.to_string();
        let command_id = command_id.to_string();

        if host_id.is_empty() || command_id.is_empty() {
            ::log::error!("Invalid parameters: {}, {}", host_id, command_id);
            return QVariantList::default();
        }

        ::log::debug!("[{}] Refreshing monitors related to command {}", host_id, command_id);

        let command = match self.command_handler.get_command_for_host(&host_id, &command_id) {
            Some(command) => command,
            None => return QVariantList::default(),
        };

        let monitor_id = command.display_options.parent_id;
        let invocation_ids = self.monitor_manager.refresh_monitors_by_id(&host_id, &monitor_id, CachePolicy::BypassCache);
        QVariantList::from_iter(invocation_ids)
    }

    fn forceRefreshMonitorsOfCategory(&mut self, host_id: QString, category: QString) -> QVariantList {
        let invocation_ids = self.monitor_manager.refresh_monitors_of_category(&host_id.to_string(), &category.to_string(), Some(CachePolicy::BypassCache));
        QVariantList::from_iter(invocation_ids)
    }

    fn getAllHostCategories(&self, host_id: QString) -> QVariantList {
        if host_id.is_empty() {
            return QVariantList::default()
        }

        self.monitor_manager.get_all_host_categories(&host_id.to_string())
                            .iter().map(|category| category.to_qvariant()).collect()
    }
}
/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

extern crate qmetaobject;
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc;

use qmetaobject::*;

use crate::command_handler::CommandButtonData;
use crate::configuration;
use crate::connection_manager::ConnectorRequest;
use crate::frontend::{self, DisplayOptions};
use crate::host_manager::StateUpdateMessage;
use crate::module::command::UIAction;
use crate::backend::CommandBackend;


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
    getCustomCommands: qt_method!(fn(&self, host_id: QString) -> QStringList),
    getCommandsOnLevel: qt_method!(fn(&self, host_id: QString, category: QString, parent_id: QString, multivalue_level: QString) -> QVariantList),
    execute: qt_method!(fn(&self, button_id: QString, host_id: QString, command_id: QString, parameters: QStringList)),
    executeConfirmed: qt_method!(fn(&self, button_id: QString, host_id: QString, command_id: QString, parameters: QStringList)),
    executePlain: qt_method!(fn(&self, host_id: QString, command_id: QString, parameters: QStringList) -> u64),
    interruptInvocation: qt_method!(fn(&self, invocation_id: u64)),
    listFiles: qt_method!(fn(&self, host_id: QString, path: QString) -> u64),
    openRemoteFileInEditor: qt_method!(fn(&self, host_id: QString, remote_path: QString)),
    saveAndUploadFile: qt_method!(fn(&self, host_id: QString, command_id: QString, remote_file_path: QString, contents: QString) -> u64),
    removeCachedFile: qt_method!(fn(&self, host_id: QString, remote_file_path: QString)),
    hasFileChanged: qt_method!(fn(&self, host_id: QString, remote_file_path: QString, contents: QString) -> bool),
    verifyHostKey: qt_method!(fn(&self, host_id: QString, connector_id: QString, key_id: QString)),

    // Host initialization methods.
    initializeHost: qt_method!(fn(&self, host_id: QString)),
    forceInitializeHost: qt_method!(fn(&self, host_id: QString)),
    forceInitializeHosts: qt_method!(fn(&self)),

    // Monitor refresh methods.
    refreshMonitorsOfCommand: qt_method!(fn(&self, host_id: QString, command_id: QString) -> QVariantList),
    refreshMonitorsOfCategory: qt_method!(fn(&self, host_id: QString, category: QString) -> QVariantList),
    refreshCertificateMonitors: qt_method!(fn(&self) -> QVariantList),

    //
    // Signals
    //

    // Signals to open a dialog. Since execution is async, invocation_id is used to retrieve the matching result.
    inputDialogOpened: qt_signal!(input_specs: QString, button_id: QString, host_id: QString, command_id: QString, parameters: QStringList),
    textDialogOpened: qt_signal!(invocation_id: u64),
    confirmationDialogOpened: qt_signal!(text: QString, button_id: QString, host_id: QString, command_id: QString, parameters: QStringList),
    commandOutputDialogOpened: qt_signal!(title: QString, invocation_id: u64),
    textViewOpened: qt_signal!(title: QString, invocation_id: u64),
    textEditorViewOpened: qt_signal!(
        header_text: QString,
        command_id: QString,
        invocation_id: u64,
        remote_file_path: QString
    ),
    terminalViewOpened: qt_signal!(header_text: QString, command: QStringList),
    fileBrowserOpened: qt_signal!(directory: QString),
    commandOutputViewOpened: qt_signal!(invocation_id: u64, title: QString, text: QString, error_text: QString, progress: u32),
    logsViewOpened: qt_signal!(time_controls: bool, title: QString, command_id: QString, parameters: QStringList, invocation_id: u64),
    commandExecuted: qt_signal!(invocation_id: u64, host_id: QString, command_id: QString, category: QString, button_identifier: QString),
    // Platform info refresh was just triggered.
    hostInitializing: qt_signal!(host_id: QString),
    error: qt_signal!(message: QString),

    //
    // Private properties
    //
    backend: Option<Box<dyn CommandBackend>>,
    configuration: configuration::Configuration,
}

#[allow(non_snake_case)]
impl CommandHandlerModel {
    pub fn new(backend: Box<dyn CommandBackend>, configuration: configuration::Configuration) -> Self {
        CommandHandlerModel {
            backend: Some(backend),
            configuration: configuration,
            ..Default::default()
        }
    }

    fn backend(&self) -> &dyn CommandBackend {
        self.backend.as_deref().unwrap()
    }

    fn backend_mut(&mut self) -> &mut dyn CommandBackend {
        self.backend.as_deref_mut().unwrap()
    }

    pub fn configure(&mut self,
        main_config: &configuration::Configuration,
        hosts_config: &configuration::Hosts,
        request_sender: mpsc::Sender<ConnectorRequest>,
        update_sender: mpsc::Sender<StateUpdateMessage>,
        frontend_update_sender: mpsc::Sender<frontend::UIUpdate>,
    ) {
        self.configuration = main_config.clone();
        self.backend_mut().configure(
            &hosts_config,
            &main_config.preferences,
            request_sender,
            update_sender,
            frontend_update_sender,
        );
    }

    pub fn start_processing_responses(&mut self) {
        self.backend_mut().start_processing_responses();
    }

    pub fn stop(&mut self) {
        self.backend_mut().stop();
    }

    pub fn refresh_host_monitors(&mut self, host_id: String) {
        self.backend_mut().refresh_host_monitors(&host_id);
    }

    fn build_editor_header_text(&self, display_options: &DisplayOptions, command_id: &str, file_path: &str) -> QString {
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(file_path);

        if !display_options.tab_title.is_empty() {
            if display_options.tab_title.contains("%s") {
                let title = display_options.tab_title.replace("%s", file_name);
                return QString::from(title);
            }

            let title = format!("{}: {}", display_options.tab_title, file_name);
            return QString::from(title);
        }
        else {
            return QString::from(command_id);
        }
    }

    // Return CommandDatas relevant to category as QVariants.
    fn getCategoryCommands(&self, host_id: QString, category: QString) -> QVariantList {
        let category_string = category.to_string();

        let mut category_commands = self.backend().commands_for_host(&host_id.to_string())
            .into_values().filter(|data| data.display_options.category == category_string)
            .collect::<Vec<CommandButtonData>>();

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

    fn getCustomCommands(&self, host_id: QString) -> QStringList {
        let custom_commands = self.backend().custom_commands_for_host(&host_id.to_string());
        custom_commands.values().map(|item| {
            match serde_json::to_string(&item) {
                Ok(json_string) => QString::from(json_string),
                Err(error) => {
                    ::log::error!("Failed to serialize: {}", error);
                    QString::from("")
                }
            }
        }).collect()
    }

    // `parent_id` is either command ID or category ID (for category-level commands).
    // Returns CommandData as JSON strings.
    fn getCommandsOnLevel(&self, host_id: QString, category: QString, parent_id: QString, multivalue_level: QString) -> QVariantList {
        let category_string = category.to_string();
        let parent_id_string = parent_id.to_string();
        let multivalue_level: u8 = match multivalue_level.to_string().parse() {
            Ok(level) => level,
            Err(_) => {
                ::log::error!("Invalid multivalue level: {}", multivalue_level);
                return QVariantList::default();
            }
        };

        let mut all_commands = self.backend().commands_for_host(&host_id.to_string())
            .into_iter().filter(|(_, data)|
                data.display_options.parent_id == parent_id_string &&
                data.display_options.category == category_string &&
                (data.display_options.multivalue_level == 0 || data.display_options.multivalue_level == multivalue_level))
            .collect::<HashMap<String, CommandButtonData>>();

        let mut valid_commands_sorted = Vec::<CommandButtonData>::new();

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
        let mut rest_of_commands: Vec<CommandButtonData> = all_commands.into_values().collect();
        rest_of_commands.sort_by(|left, right| left.command_id.cmp(&right.command_id));
        valid_commands_sorted.append(&mut rest_of_commands);

        // Return list of JSONs.
        valid_commands_sorted.iter()
            .map(|item| serde_json::to_string(&item).unwrap_or_default().to_qvariant())
            .collect()
    }

    fn execute(&mut self, button_id: QString, host_id: QString, command_id: QString, parameters: QStringList) {
        let display_options = match self.backend().command_for_host(&host_id.to_string(), &command_id.to_string()) {
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

        let display_options = match self.backend().command_for_host(&host_id, &command_id) {
            Some(command_data) => command_data.display_options,
            None => return,
        };

        match display_options.action {
            UIAction::None => {
                let invocation_id = self.backend_mut().execute_command(&host_id, &command_id, &parameters);

                if invocation_id > 0 {
                    self.commandExecuted(invocation_id, host_id.into(), command_id.into(), display_options.category.into(), button_id.into());
                }
            },
            UIAction::FollowOutput => {
                let invocation_id = self.backend_mut().execute_command(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    let title = match display_options.tab_title.is_empty() {
                        true => QString::from(format!("{}: {}", command_id, parameters.first().unwrap_or(&String::new()))),
                        false => QString::from(display_options.tab_title)
                    };
                    self.commandOutputDialogOpened(title, invocation_id);
                    self.commandExecuted(invocation_id, host_id.into(), command_id.into(), display_options.category.into(), button_id.into());
                }
            },
            UIAction::TextView => {
                let target_id = parameters.first().unwrap().clone();
                let invocation_id = self.backend_mut().execute_command(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    self.textViewOpened(QString::from(format!("{}: {}", command_id, target_id)), invocation_id)
                }
            },
            UIAction::TextDialog => {
                let invocation_id = self.backend_mut().execute_command(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    self.textDialogOpened(invocation_id)
                }
            },
            UIAction::LogView => {
                let invocation_id = self.backend_mut().execute_command(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    let parameters_qs = parameters.into_iter().map(QString::from).collect::<QStringList>();
                    self.logsViewOpened(false, QString::from(display_options.tab_title), QString::from(command_id), parameters_qs, invocation_id);
                }
            },
            UIAction::LogViewWithTimeControls => {
                let invocation_id = self.backend_mut().execute_command(&host_id, &command_id, &parameters);
                if invocation_id > 0 {
                    let parameters_qs = parameters.into_iter().map(QString::from).collect::<QStringList>();
                    self.logsViewOpened(true, QString::from(display_options.tab_title), QString::from(command_id), parameters_qs, invocation_id);
                }
            },
            UIAction::Terminal => {
                let Some(local_backend) = self.backend().local_backend() else {
                    return;
                };

                if self.configuration.preferences.terminal == configuration::INTERNAL {
                    let command = local_backend.remote_terminal_command(&host_id, &command_id, &parameters);
                    let command_qsl = command.to_vec().into_iter().map(QString::from).collect::<QStringList>();
                    self.terminalViewOpened(QString::from(display_options.tab_title), command_qsl)
                }
                else {
                    local_backend.open_external_terminal(&host_id, &command_id, parameters);
                }
            },
            UIAction::FileBrowser => {
                let directory = match parameters.first() {
                    Some(directory) => directory.clone().into(),
                    None => "/".to_string().into(),
                };

                self.fileBrowserOpened(directory)
            },
            UIAction::TextEditor => {
                // Commands with parent monitors receive file path as a parameter, but category-level
                // commands don't. They should return it in `get_connector_message()`.
                let remote_file_path = if let Some(path) =
                    self.backend_mut().resolve_text_editor_path(&host_id, &command_id, &parameters)
                {
                    path
                } else {
                    return;
                };

                if self.configuration.preferences.use_remote_editor {
                    let Some(local_backend) = self.backend().local_backend() else {
                        return;
                    };

                    if self.configuration.preferences.terminal == configuration::INTERNAL {
                        let command = local_backend.remote_text_editor_command(&host_id, &remote_file_path);
                        let command_qsl = command.to_vec().into_iter().map(QString::from).collect::<QStringList>();
                        let editor_header_text = self.build_editor_header_text(&display_options, &command_id, &remote_file_path);
                        self.terminalViewOpened(editor_header_text, command_qsl);
                    }
                    else {
                        local_backend.open_external_terminal(&host_id, &command_id, parameters);
                    }
                }
                else {
                    if self.configuration.preferences.text_editor == configuration::INTERNAL 
                        || self.configuration.preferences.text_editor == configuration::INTERNAL_SIMPLE {

                        let (invocation_id, _) =
                            self.backend_mut().download_editable_file(&host_id, &command_id, &remote_file_path);
                        let editor_header_text = self.build_editor_header_text(&display_options, &command_id, &remote_file_path);
                        self.textEditorViewOpened(
                            editor_header_text,
                            QString::from(command_id.clone()),
                            invocation_id,
                            QString::from(remote_file_path.clone()),
                        );
                    }
                    else {
                        let Some(local_backend) = self.backend().local_backend() else {
                            return;
                        };
                        let local_file_path = local_backend.open_external_text_editor(
                            &host_id,
                            &command_id,
                            &remote_file_path,
                        );
                        let _invocation_id = self.backend_mut().upload_file(&host_id, &command_id, &local_file_path);
                    }
                }
            },
        }
    }

    fn executePlain(&mut self, host_id: QString, command_id: QString, parameters: QStringList) -> u64 {
        let host_id = host_id.to_string();
        let command_id = command_id.to_string();
        let parameters: Vec<String> = parameters.into_iter().map(|qvar| qvar.to_string()).collect();
        self.backend_mut().execute_command(&host_id, &command_id, &parameters)
    }

    fn interruptInvocation(&self, invocation_id: u64) {
        self.backend().interrupt_invocation(invocation_id);
    }

    fn listFiles(&mut self, host_id: QString, path: QString) -> u64 {
        let host_id = host_id.to_string();
        let parameters = vec![path.to_string()];
        let command_id = String::from("_internal-filebrowser-ls");
        let invocation_id = self.backend_mut().execute_command(&host_id, &command_id, &parameters);

        invocation_id
    }

    fn openRemoteFileInEditor(&mut self, host_id: QString, remote_path: QString) {
        let host_id = host_id.to_string();
        let remote_path = remote_path.to_string();
        let command_id = String::from("_internal-filebrowser-edit");
        let display_options = match self.backend().command_for_host(&host_id, &command_id) {
            Some(command_data) => command_data.display_options,
            None => return,
        };
        let (invocation_id, _) =
            self.backend_mut().download_editable_file(&host_id, &command_id, &remote_path);
        if invocation_id > 0 {
            let editor_header_text = self.build_editor_header_text(&display_options, &command_id, &remote_path);
            self.textEditorViewOpened(
                editor_header_text,
                QString::from(command_id.clone()),
                invocation_id,
                QString::from(remote_path.clone()),
            );
        }
    }

    fn saveAndUploadFile(&mut self, host_id: QString, command_id: QString, remote_file_path: QString, contents: QString) -> u64 {
        let host_id = host_id.to_string();
        let command_id = command_id.to_string();
        let remote_file_path = remote_file_path.to_string();
        let contents = contents.to_string().into_bytes();

        if self.backend().local_backend().is_some() {
            self.backend_mut().write_cached_file(&host_id, &remote_file_path, contents);
            self.backend_mut().upload_file_from_cache(&host_id, &command_id, &remote_file_path)
        }
        else {
            self.backend_mut().upload_file_from_editor(&host_id, &command_id, &remote_file_path, contents)
        }
    }

    fn removeCachedFile(&mut self, host_id: QString, remote_file_path: QString) {
        let host_id = host_id.to_string();
        let remote_file_path = remote_file_path.to_string();
        self.backend_mut().remove_cached_file(&host_id, &remote_file_path);
    }

    fn hasFileChanged(&self, host_id: QString, remote_file_path: QString, contents: QString) -> bool {
        let host_id = host_id.to_string();
        let remote_file_path = remote_file_path.to_string();
        let contents = contents.to_string().into_bytes();
        self.backend()
            .has_cached_file_changed(&host_id, &remote_file_path, &contents)
    }

    fn verifyHostKey(&self, host_id: QString, connector_id: QString, key_id: QString) {
        let host_id = host_id.to_string();
        let connector_id = connector_id.to_string();
        let key_id = key_id.to_string();
        self.backend().verify_host_key(&host_id, &connector_id, &key_id);
    }

    fn check_config_errors(&self) -> bool {
        if self.configuration.config_errors.is_empty() {
            return false;
        }

        for error in &self.configuration.config_errors {
            let message = format!("Configuration error: {}", error);
            self.error(QString::from(message));
        }

        true
    }

    fn initializeHost(&mut self, host_id: QString) {
        if self.check_config_errors() {
            return;
        }

        self.backend_mut().initialize_host(&host_id.to_string());
        self.hostInitializing(host_id);
    }

    fn forceInitializeHost(&mut self, host_id: QString) {
        if self.check_config_errors() {
            return;
        }

        self.backend_mut().initialize_host(&host_id.to_string());
        self.hostInitializing(host_id);
    }

    fn forceInitializeHosts(&mut self) {
        if self.check_config_errors() {
            return;
        }

        let host_ids = self.backend_mut().initialize_hosts();
        for host_id in host_ids {
            self.hostInitializing(QString::from(host_id));
        }
    }

    // Finds related monitors for a command and refresh them.
    fn refreshMonitorsOfCommand(&mut self, host_id: QString, command_id: QString) -> QVariantList  {
        if self.check_config_errors() {
            return QVariantList::default();
        }

        let host_id = host_id.to_string();
        let command_id = command_id.to_string();

        if host_id.is_empty() || command_id.is_empty() {
            ::log::error!("Invalid parameters: {}, {}", host_id, command_id);
            return QVariantList::default();
        }

        ::log::debug!("[{}] Refreshing monitors related to command {}", host_id, command_id);

        match self.backend().command_for_host(&host_id, &command_id) {
            Some(_) => {}
            None => return QVariantList::default(),
        }

        let invocation_ids = self.backend_mut().refresh_monitors_for_command(&host_id, &command_id);

        QVariantList::from_iter(invocation_ids)
    }

    fn refreshMonitorsOfCategory(&mut self, host_id: QString, category: QString) -> QVariantList {
        if self.check_config_errors() {
            return QVariantList::default();
        }

        let invocation_ids = self.backend_mut().refresh_monitors_of_category(&host_id.to_string(), &category.to_string());
        QVariantList::from_iter(invocation_ids)
    }

    fn refreshCertificateMonitors(&mut self) -> QVariantList {
        if self.check_config_errors() {
            return QVariantList::default();
        }

        let invocation_ids = self.backend_mut().refresh_certificate_monitors();
        QVariantList::from_iter(invocation_ids)
    }

    fn getAllHostCategories(&self, host_id: QString) -> QVariantList {
        if host_id.is_empty() {
            return QVariantList::default()
        }

        self.backend().all_host_categories(&host_id.to_string())
                            .iter().map(|category| category.to_qvariant()).collect()
    }
}
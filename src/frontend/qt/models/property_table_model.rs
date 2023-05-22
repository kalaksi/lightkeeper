
extern crate qmetaobject;
use qmetaobject::*;
use serde_derive::Serialize;
use std::collections::HashMap;

use crate::command_handler::CommandData;
use crate::enums::Criticality;
use crate::{frontend, configuration};
use crate::module::monitoring::{DataPoint, MonitoringData};


const SEPARATOR_TOKEN: &str = "sep";
const COOLDOWN_LENGTH: u32 = 15000;

#[derive(QObject, Default)]
pub struct PropertyTableModel {
    base: qt_base_class!(trait QAbstractTableModel),

    // Monitoring and command data properties.
    monitoring_datas: qt_property!(QVariantList; WRITE set_monitoring_datas),
    command_datas: qt_property!(QVariantList; WRITE set_command_datas),
    display_options: qt_property!(QVariant; WRITE set_display_options),

    // init: qt_method!(fn(&mut self, monitoring_datas: QVariantList, command_datas: QVariantList)),
    update: qt_method!(fn(&mut self, monitoring_data: QVariant)),
    get_separator_label: qt_method!(fn(&mut self, row: QVariant) -> QString),
    get_row_height: qt_method!(fn(&mut self, row: QVariant) -> u32),
    get_column_width: qt_method!(fn(&self, row: QVariant, column: QVariant) -> f32),

    // For command cooldown mechanism.
    // State has to be stored and handled here and not in CommandButton or CommandButtonRow since table content isn't persistent.
    start_command_cooldown: qt_method!(fn(&mut self, button_identifier: QString, invocation_id: u64)),
    decrease_command_cooldowns: qt_method!(fn(&mut self, cooldown_decrement: u32) -> u32),
    end_command_cooldown: qt_method!(fn(&mut self, invocation_id: u64)),
    get_command_cooldowns: qt_method!(fn(&self, row: u32) -> QString),


    // Internal data structures.
    i_monitoring_datas: Vec<MonitoringData>,
    i_command_datas: Vec<CommandData>,
    i_display_options: configuration::DisplayOptions,
    command_cooldown_times: HashMap<String, u32>,
    command_cooldowns_finishing: Vec<String>,
    command_invocations: HashMap<u64, String>,

    /// Holds preprocessed data more fitting for table rows.
    row_datas: Vec<RowData>,
}

impl PropertyTableModel {
    /// Updates monitoring data. Expects MonitoringData as QVariant.
    pub fn update(&mut self, new_data: QVariant) {
        let mut new_data = MonitoringData::from_qvariant(new_data).unwrap();

        self.begin_reset_model();
        if let Some(old_data) = self.i_monitoring_datas.iter_mut().find(|old_data| old_data.monitor_id == new_data.monitor_id) {
            std::mem::swap(old_data, &mut new_data);
        }
        else {
            self.i_monitoring_datas.push(new_data);
        }

        self.update_row_datas();
        self.end_reset_model();
    }

    fn set_monitoring_datas(&mut self, monitoring_datas: QVariantList) {
        self.begin_reset_model();
        self.i_monitoring_datas = monitoring_datas.into_iter().map(|qv| MonitoringData::from_qvariant(qv.clone()).unwrap()).collect();
        self.update_row_datas();
        self.end_reset_model();
    }

    fn set_command_datas(&mut self, command_datas: QVariantList) {
        self.begin_reset_model();
        self.i_command_datas = command_datas.into_iter().map(|qv| CommandData::from_qvariant(qv.clone()).unwrap()).collect();
        self.update_row_datas();
        self.end_reset_model();
    }

    fn set_display_options(&mut self, display_options: QVariant) {
        self.begin_reset_model();
        self.i_display_options = configuration::DisplayOptions::from_qvariant(display_options).unwrap();
        self.update_row_datas();
        self.end_reset_model();
    }

    fn update_row_datas(&mut self) {
        let mut row_datas = self.i_monitoring_datas.iter().flat_map(|m_data| Self::convert_to_row_data(&m_data, &self.i_command_datas)).collect();
        self.sort_row_data(&mut row_datas);
        Self::insert_multivalue_separator_rows(&mut row_datas);
        self.row_datas = row_datas;
    }

    fn get_separator_label(&mut self, row: QVariant) -> QString {
        let row = usize::from_qvariant(row).unwrap();

        if let Some(row_data) = self.row_datas.get(row) {
            if row_data.value.value == SEPARATOR_TOKEN {
                return QString::from(row_data.display_options.display_text.clone());
            }
        }
        QString::from("")
    }

    // Higher row if description is present.
    fn get_row_height(&mut self, row: QVariant) -> u32 {
        let row = usize::from_qvariant(row).unwrap();
        if let Some(row_data) = self.row_datas.get(row) {
            if row_data.value.value == SEPARATOR_TOKEN {
                50
            }
            // If any row has description set, use a higher row height.
            else if self.row_datas.iter().any(|row_data| !row_data.value.description.is_empty()) {
                30
            }
            else {
                22
            }
        }
        else {
            0
        }
    }

    fn get_column_width(&self, row: QVariant, column: QVariant) -> f32 {
        let row = usize::from_qvariant(row).unwrap();
        let column = usize::from_qvariant(column).unwrap();
        let width_for_commands = 0.175;

        if let Some(row_data) = self.row_datas.get(row) {
            match column {
                0 => 0.48,
                1 => {
                    if row_data.command_datas.len() > 0 {
                        0.52 - width_for_commands
                    }
                    else {
                        0.52
                    }
                },
                2 => {
                    if row_data.command_datas.len() > 0 {
                        width_for_commands
                    }
                    // Hidden if there are no related commands.
                    else {
                        0.0
                    }
                }
                _ => 0.0,
            }
        }
        else {
            0.0
        }
    }

    // Practically flattens multivalue data and does some filtering.
    fn convert_to_row_data(monitoring_data: &MonitoringData, command_datas: &Vec<CommandData>) -> Vec<RowData> {
        let mut row_datas = Vec::<RowData>::new();
        let last_data_point = monitoring_data.values.iter().last().unwrap();

        if monitoring_data.display_options.use_multivalue {

            for multivalue1 in last_data_point.multivalue.iter() {
                if let Some(row_data) = Self::create_single_row_data(monitoring_data, multivalue1.clone(), 1, command_datas) {
                    row_datas.push(row_data);
                }

                for multivalue2 in multivalue1.multivalue.iter() {
                    if let Some(row_data) = Self::create_single_row_data(monitoring_data, multivalue2.clone(), 2, command_datas) {
                        row_datas.push(row_data);
                    }
                }
            }
        }
        else {
            if let Some(row_data) = Self::create_single_row_data(monitoring_data, last_data_point.clone(), 0, command_datas) {
                row_datas.push(row_data);
            }
        }

        return row_datas;
    }

    fn sort_row_data(&self, row_datas: &mut Vec<RowData>) {
        if row_datas.is_empty() {
            return;
        }

        let category = &row_datas.first().unwrap().display_options.category;
        let monitor_order = match &self.i_display_options.categories.get(category) {
            Some(category_data) => category_data.monitor_order.clone().unwrap_or_default(),
            None => Vec::new(),
        };

        // Orders first by predefined order and then alphabetically.
        row_datas.sort_by_key(|row_data| {
            // Priority will be the position in the predefined order or (shared) last priority if not found.
            let priority = monitor_order.iter().position(|id| id == &row_data.monitor_id)
                                               .unwrap_or(monitor_order.len());

            // Tuple for sorting by priority and then by name.
            (priority, row_data.monitor_id.clone())
        });
    }

    fn create_single_row_data(monitoring_data: &MonitoringData, mut data_point: DataPoint, multivalue_level: u8,
                              command_datas: &Vec<CommandData>) -> Option<RowData> {
        if data_point.criticality == Criticality::Ignore {
            return None;
        }

        // Find commands relevant to this row and populate command.parameters property from data point.
        let level_commands = command_datas.iter()
            .filter(|command| command.display_options.parent_id == monitoring_data.monitor_id &&
                              (command.display_options.multivalue_level == 0 ||
                               command.display_options.multivalue_level == multivalue_level))
            .map(|command| {
                let mut full_command = command.clone();
                full_command.command_params = data_point.command_params.clone();
                full_command
            })
            .collect::<Vec<CommandData>>();

        if multivalue_level > 1 {
            // TODO: proper padding with QML instead of spaces.
            let indent = "    ".repeat((multivalue_level - 1).into());
            data_point.label = format!("{}{}", indent, data_point.label);
            if !data_point.description.is_empty() {
                let indent_d = "      ".repeat((multivalue_level - 1).into());
                data_point.description = format!("{}{}", indent_d, data_point.description);
            }
        }

        Some(RowData {
            monitor_id: monitoring_data.monitor_id.clone(),
            value: data_point,
            display_options: monitoring_data.display_options.clone(),
            command_datas: level_commands,
        })
    }

    // Adds a special table row for labeled separators before multivalue monitoring data.
    fn insert_multivalue_separator_rows(row_data: &mut Vec<RowData>) {
        // Iterate backwards so that indices don't get messed up when inserting.
        for i in (0..row_data.len()).rev() {
            // Older behavior:
            // Insert separator between multivalue and non-multivalue data, but not on the first row.
            // if row_data[i].display_options.use_multivalue &&
            //    (i > 0 && row_data[i-1].display_options.use_multivalue == false) {

            if i > 0 &&
               row_data[i].display_options.use_multivalue &&
               row_data[i-1].monitor_id != row_data[i].monitor_id {

                let separator_row = RowData {
                    value: DataPoint {
                        value: String::from(SEPARATOR_TOKEN),
                        criticality: Criticality::Ignore,
                        ..Default::default()
                    },
                    display_options: frontend::DisplayOptions {
                        display_text: row_data[i].display_options.display_text.clone(),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                row_data.insert(i, separator_row);
            }
        }
    }

    fn start_command_cooldown(&mut self, button_identifier: QString, invocation_id: u64) {
        let button_identifier = button_identifier.to_string();
        self.command_cooldown_times.insert(button_identifier.clone(), COOLDOWN_LENGTH);
        self.command_invocations.insert(invocation_id, button_identifier);
    }

    fn decrease_command_cooldowns(&mut self, cooldown_decrement: u32) -> u32 {
        for (button_identifier, cooldown_time) in self.command_cooldown_times.iter_mut() {
            // Quickly decrease cooldown if command is finished.
            let actual_decrement = match self.command_cooldowns_finishing.contains(button_identifier) {
                true => 20 * cooldown_decrement,
                false => cooldown_decrement,
            };

            if actual_decrement > *cooldown_time {
                *cooldown_time = 0;
                self.command_cooldowns_finishing.retain(|c| c != button_identifier);
            }
            else {
                *cooldown_time -= actual_decrement;
            };
        }

        self.command_cooldown_times.retain(|_, cooldown_time| *cooldown_time > 0);
        self.command_cooldown_times.len() as u32
    }

    fn end_command_cooldown(&mut self, invocation_id: u64) {
        // Does nothing if the invocation_id doesn't belong to this table instance.
        if let Some(button_identifier) = self.command_invocations.remove(&invocation_id) {
            self.command_cooldowns_finishing.push(button_identifier);
        }
    }

    fn get_command_cooldowns(&self, row: u32) -> QString {
        let row_data = self.row_datas.get(row as usize).unwrap().clone();
        let command_datas = row_data.command_datas;

        let mut cooldowns = HashMap::<String, f32>::new();
        for command_data in command_datas {
            let button_identifier = format!("{}{}", command_data.command_id, command_data.command_params.join(""));
            let remaining_time = self.command_cooldown_times.get(&button_identifier).unwrap_or(&0);
            let remaining_percentage = *remaining_time as f32 / COOLDOWN_LENGTH as f32;
            cooldowns.insert(button_identifier, remaining_percentage);
        }

        QString::from(serde_json::to_string(&cooldowns).unwrap())
    }
}


impl QAbstractTableModel for PropertyTableModel {
    fn row_count(&self) -> i32 {
        self.row_datas.len() as i32
    }

    fn column_count(&self) -> i32 {
        3
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        if role != USER_ROLE {
            return QString::from(format!("Unknown role: {}", role)).to_qvariant();
        }

        let row_data = self.row_datas.get(index.row() as usize).unwrap().clone();
 
        // TODO: avoid JSON encoding?
        match index.column() {
            0 => {
                let label = match row_data.display_options.use_multivalue {
                    true => row_data.value.label.clone(),
                    false => row_data.display_options.display_text.clone()
                };
                let label_with_description = LabelAndDescription {
                    label: label,
                    description: row_data.value.description.clone()
                };
                let label_with_description_json = serde_json::to_string(&label_with_description).unwrap();
                label_with_description_json.to_qvariant()
            },
            1 => {
                let styled_value = StyledValue {
                    data_point: row_data.value.clone(),
                    display_options: row_data.display_options.clone()
                };

                let styled_value_json = serde_json::to_string(&styled_value).unwrap();
                styled_value_json.to_qvariant()
            },
            2 => {
                // Filters out commands that depend on specific criticality, value or tag that isn't present currently.
                let command_datas = row_data.command_datas.into_iter()
                    .filter(|command| command.display_options.depends_on_criticality.is_empty() ||
                                    command.display_options.depends_on_criticality.contains(&row_data.value.criticality))
                    .filter(|command| command.display_options.depends_on_value.is_empty() ||
                                    command.display_options.depends_on_value.contains(&row_data.value.value))
                    .filter(|command| command.display_options.depends_on_tags.iter().all(|tag| row_data.value.tags.contains(tag)))
                    .filter(|command| command.display_options.depends_on_no_tags.iter().all(|tag| !row_data.value.tags.contains(tag)))
                    .collect::<Vec<CommandData>>();

                serde_json::to_string(&command_datas).unwrap().to_qvariant()
            },
            _ => panic!(),
        }
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        vec![(USER_ROLE, QByteArray::from("value"))].into_iter().collect()
    }
}


#[derive(Default, Clone, Serialize)]
struct RowData {
    monitor_id: String,
    value: DataPoint,
    display_options: frontend::DisplayOptions,
    command_datas: Vec<CommandData>,
}

#[derive(Default, Clone, Serialize)]
struct StyledValue {
    data_point: DataPoint,
    display_options: frontend::DisplayOptions,
}


#[derive(Default, Clone, Serialize)]
struct LabelAndDescription {
    label: String,
    description: String,
}
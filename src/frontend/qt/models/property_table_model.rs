
extern crate qmetaobject;

use qmetaobject::*;
use serde_derive::Serialize;

use crate::command_handler::CommandData;
use crate::enums::Criticality;
use crate::frontend;
use crate::module::monitoring::{DataPoint, MonitoringData};


const SEPARATOR_TOKEN: &str = "sep";

#[derive(QObject, Default)]
pub struct PropertyTableModel {
    base: qt_base_class!(trait QAbstractTableModel),

    // Monitoring and command data properties.
    monitoring_datas: qt_property!(QVariantList; WRITE set_monitoring_datas),
    command_datas: qt_property!(QVariantList; WRITE set_command_datas),

    // init: qt_method!(fn(&mut self, monitoring_datas: QVariantList, command_datas: QVariantList)),
    update: qt_method!(fn(&mut self, monitoring_data: QVariant)),

    get_separator_label: qt_method!(fn(&mut self, row: QVariant) -> QString),

    // Internal data structures.
    i_monitoring_datas: Vec<MonitoringData>,
    i_command_datas: Vec<CommandData>,
    /// Holds preprocessed data more fitting for table rows.
    row_datas: Vec<RowData>
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

        let mut row_datas: Vec<RowData> = self.i_monitoring_datas.iter().flat_map(|m_data| Self::convert_to_row_data(&m_data, &self.i_command_datas))
                                                                 .collect();
        Self::insert_multivalue_separator_rows(&mut row_datas);
        self.row_datas = row_datas;

        self.end_reset_model();
    }

    fn set_monitoring_datas(&mut self, monitoring_datas: QVariantList) {
        self.begin_reset_model();
        self.i_monitoring_datas = monitoring_datas.into_iter().map(|qv| MonitoringData::from_qvariant(qv.clone()).unwrap()).collect();

        let mut row_datas: Vec<RowData> = self.i_monitoring_datas.iter().flat_map(|m_data| Self::convert_to_row_data(&m_data, &self.i_command_datas))
                                                                 .collect();
        Self::insert_multivalue_separator_rows(&mut row_datas);
        self.row_datas = row_datas;
        self.end_reset_model();
    }

    fn set_command_datas(&mut self, command_datas: QVariantList) {
        self.begin_reset_model();
        self.i_command_datas = command_datas.into_iter().map(|qv| CommandData::from_qvariant(qv.clone()).unwrap()).collect();

        let mut row_datas = self.i_monitoring_datas.iter().flat_map(|m_data| Self::convert_to_row_data(&m_data, &self.i_command_datas)).collect();
        Self::insert_multivalue_separator_rows(&mut row_datas);
        self.row_datas = row_datas;

        self.end_reset_model();
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
            let indent = "    ".repeat((multivalue_level - 1).into());
            data_point.label = format!("{}{}", indent, data_point.label);
        }

        Some(RowData {
            value: data_point,
            display_options: monitoring_data.display_options.clone(),
            command_datas: level_commands,
        })
    }

    // Adds a special table row for labeled separators between non-multivalue and multivalue monitoring data.
    fn insert_multivalue_separator_rows(row_data: &mut Vec<RowData>) {
        // Iterate backwards so that indices don't get messed up when inserting.
        for i in (0..row_data.len()).rev() {
            // Insert separator between multivalue and non-multivalue data, but not on the first row.
            if row_data[i].display_options.use_multivalue &&
               (i > 0 && row_data[i-1].display_options.use_multivalue == false) {

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
        let styled_value = StyledValue {
            data_point: row_data.value.clone(),
            display_options: row_data.display_options.clone()
        };

        let label = if row_data.display_options.use_multivalue {
            row_data.value.label.clone()
        }
        else {
            row_data.display_options.display_text.clone()
        };
        let styled_value_json = serde_json::to_string(&styled_value).unwrap();

        match index.column() {
            0 => label.to_qvariant(),
            1 => styled_value_json.to_qvariant(),
            // TODO: Maybe avoid using JSON encoding here too?
            2 => serde_json::to_string(&row_data.command_datas).unwrap().to_qvariant(),
            _ => panic!(),
        }
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        vec![(USER_ROLE, QByteArray::from("value"))].into_iter().collect()
    }
}


#[derive(Default, Clone, Serialize)]
struct RowData {
    value: DataPoint,
    display_options: frontend::DisplayOptions,
    command_datas: Vec<CommandData>,
}

#[derive(Default, Clone, Serialize)]
struct StyledValue {
    data_point: DataPoint,
    display_options: frontend::DisplayOptions,
}
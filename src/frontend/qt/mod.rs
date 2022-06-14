use std::str::FromStr;
use std::cell::RefCell;
extern crate qmetaobject;
use cstr::cstr;
use qmetaobject::*;

use crate::utils::enums::HostStatus;
use super::{ Frontend, DisplayData };
use crate::module::monitoring::{
    Criticality,
    DataPoint,
    DisplayOptions,
    DisplayStyle
};

pub struct QmlFrontend;

impl QmlFrontend {
    pub fn draw(display_data: &DisplayData) {
        let table_ref = RefCell::new(Table {
            data: Vec::new(),
            ..Default::default()
        });

        {
            let mut table = table_ref.borrow_mut();

            for (_, host_data) in display_data.hosts.iter() {
                table.data.push(TableEntry {
                    name: host_data.name.clone().into(),
                    fqdn: host_data.domain_name.clone().into(),
                    ip_address: host_data.ip_address.to_string().into(),
                    status: host_data.status.to_string().into(),
                });
            }
        }

        // qml_register_type::<Table>(cstr!("Lightkeeper"), 0, 1, cstr!("Lightkeeper"));

        let mut engine = QmlEngine::new();
        engine.set_object_property("_model".into(), unsafe { QObjectPinned::new(&table_ref) });
        engine.load_file(QString::from("src/frontend/qt/main.qml"));
        engine.exec();
    }
}



#[derive(QObject, Default)]
struct Table {
    base: qt_base_class!(trait QAbstractListModel),
    data: Vec<TableEntry>,
}

impl QAbstractListModel for Table {
    fn row_count(&self) -> i32 {
        self.data.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        if role != USER_ROLE {
            return QVariant::default();
        }
        self.data.get(index.row() as usize).map(|x| x.to_qvariant()).unwrap_or_default()
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        vec![(USER_ROLE, QByteArray::from("value"))].into_iter().collect()
    }
}

#[derive(QGadget, Clone, Default)]
struct TableEntry {
    pub name: qt_property!(QString),
    pub fqdn: qt_property!(QString),
    pub ip_address: qt_property!(QString),
    pub status: qt_property!(QString),
}


fn convert_to_display_string(data_point: &DataPoint, display_options: &DisplayOptions) -> String {
    if data_point.is_empty() {
        if data_point.criticality == Criticality::Critical {
            String::from("Error")
        }
        else {
            String::from("")
        }
    }
    else if display_options.use_multivalue {
        convert_multivalue(data_point, display_options)
    }
    else {
        convert_single(data_point, display_options)
    }
}

fn convert_single(data_point: &DataPoint, display_options: &DisplayOptions) -> String {
    let single_value = match display_options.display_style {
        DisplayStyle::CriticalityLevel => {
            match data_point.criticality {
                Criticality::NoData => String::from("No data"),
                Criticality::Normal => String::from("Normal"),
                Criticality::Warning => String::from("Warning"),
                Criticality::Error => String::from("Error"),
                Criticality::Critical => String::from("Critical"),
            }
        },
        DisplayStyle::StatusUpDown => {
            match HostStatus::from_str(&data_point.value).unwrap_or_default() {
                HostStatus::Up => "Up".to_string(),
                HostStatus::Down => "Down".to_string(),
            }
        },
        DisplayStyle::String => {
            data_point.value.to_string()
        },
    };

    single_value
}

fn convert_multivalue(data_point: &DataPoint, display_options: &DisplayOptions) -> String {
    let mut separator = ", ";

    // Process all values and join them into string in the end.
    data_point.multivalue.iter().map(|data_point| {
        let single_value = match display_options.display_style {
            DisplayStyle::CriticalityLevel => {
                separator = "";

                match data_point.criticality {
                    Criticality::NoData => "No data".to_string(),
                    Criticality::Normal => "▩".to_string(),
                    Criticality::Warning =>"▩".to_string(),
                    Criticality::Error => "▩".to_string(),
                    Criticality::Critical =>"▩".to_string(),
                }
            },
            DisplayStyle::StatusUpDown => {
                match HostStatus::from_str(&data_point.value).unwrap_or_default() {
                    HostStatus::Up => "Up".to_string(),
                    HostStatus::Down => "Down".to_string(),
                }
            },
            DisplayStyle::String => {
                data_point.value.to_string()
            },
        };

        single_value
    }).collect::<Vec<String>>().join(separator)
}
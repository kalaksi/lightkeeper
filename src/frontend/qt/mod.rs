use std::{str::FromStr, default};
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
        qmetaobject::log::init_qt_to_rust();

        let qt_data = QObjectBox::new(Table::from(&display_data));

        // qml_register_type::<Table>(cstr!("Lightkeeper"), 0, 1, cstr!("Lightkeeper"));

        let mut engine = QmlEngine::new();
        engine.set_object_property("lightkeeper_data".into(), qt_data.pinned());
        engine.load_file(QString::from("src/frontend/qt/qml/main.qml"));
        engine.exec();
    }
}


#[derive(QObject, Default)]
struct Table {
    base: qt_base_class!(trait QAbstractTableModel),
    headers: Vec<QString>,
    data: Vec<Vec<QString>>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            headers: Vec::new(),
            data: Vec::new(),
            ..Default::default()
        }
    }

    pub fn from(display_data: &DisplayData) -> Self {
        let mut table_data = Table::new();

        for header in &display_data.table_headers {
            table_data.headers.push(header.clone().into());
        }

        for (_, host_data) in display_data.hosts.iter() {
            let host_status = convert_to_display_string(&DataPoint::new(host_data.status.to_string()),
                                                        &DisplayOptions::just_style(DisplayStyle::StatusUpDown));

            let mut row: Vec<QString> = vec![ host_status.to_string().into(),
                                             host_data.name.clone().into(),
                                             host_data.domain_name.clone().into(),
                                             host_data.ip_address.to_string().into() ];

            for monitor_id in &display_data.all_monitor_names {
                match host_data.monitoring_data.get(monitor_id) {
                    // There should always be some monitoring data if the key exists.
                    Some(monitoring_data) => row.push(convert_to_display_string(monitoring_data.values.last().unwrap(),
                                                                                &monitoring_data.display_options)),
                    None => row.push(QString::from(""))
                }
            }

            table_data.data.push(row);
        }

        table_data
    }
}


impl QAbstractTableModel for Table {
    fn row_count(&self) -> i32 {
        self.data.len() as i32
    }

    fn column_count(&self) -> i32 {
        self.headers.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        if role != USER_ROLE {
            return QVariant::default();
        }

        let row = self.data.get(index.row() as usize).unwrap();
        row.get(index.column() as usize).map(|value| value.to_qvariant()).unwrap_or_default()
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        vec![(USER_ROLE, QByteArray::from("value"))].into_iter().collect()
    }
}

#[derive(QGadget, Clone, Default)]
struct ColoredText {
    pub text: qt_property!(QString),
    pub color: qt_property!(QString),
}

fn color_by_level(text: String, criticality: Criticality) -> QString {
    match criticality {
        Criticality::NoData => QString::from(""),
        Criticality::Normal => QString::from("green"),
        Criticality::Warning => QString::from("yellow"),
        Criticality::Error => QString::from("red"),
        Criticality::Critical => QString::from("red"),
    }
}


fn convert_to_display_string(data_point: &DataPoint, display_options: &DisplayOptions) -> QString {
    if data_point.is_empty() {
        if data_point.criticality == Criticality::Critical {
            QString::from("Error")
        }
        else {
            QString::from("")
        }
    }
    else if display_options.use_multivalue {
        convert_multivalue(data_point, display_options)
    }
    else {
        convert_single(data_point, display_options)
    }
}

fn convert_single(data_point: &DataPoint, display_options: &DisplayOptions) -> QString {
    let single_value = match display_options.display_style {
        DisplayStyle::CriticalityLevel => {
            match data_point.criticality {
                Criticality::NoData => QString::from("No data"),
                Criticality::Normal => QString::from("Normal"),
                Criticality::Warning => QString::from("Warning"),
                Criticality::Error => QString::from("Error"),
                Criticality::Critical => QString::from("Critical"),
            }
        },
        DisplayStyle::StatusUpDown => {
            match HostStatus::from_str(&data_point.value).unwrap_or_default() {
                HostStatus::Up => QString::from("Up"),
                HostStatus::Down => QString::from("Down"),
            }
        },
        DisplayStyle::String => {
            QString::from(data_point.value.clone())
        },
    };

    single_value
}

fn convert_multivalue(data_point: &DataPoint, display_options: &DisplayOptions) -> QString {
    let mut separator = ", ";

    // Process all values and join them into string in the end.
    let display_value = data_point.multivalue.iter().map(|data_point| {
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
                    HostStatus::Up => String::from("Up"),
                    HostStatus::Down => String::from("Down"),
                }
            },
            DisplayStyle::String => {
                data_point.value.to_string()
            },
        };

        single_value
    }).collect::<Vec<String>>();

    QString::from(display_value.join(separator))
}
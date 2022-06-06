
use std::str::FromStr;
use owo_colors::OwoColorize;
use std::net::IpAddr;
use tabled::{ Tabled, Style, builder::Builder };
use super::{ Frontend, DisplayData };
use crate::{module::monitoring::{Criticality, DataPoint, DisplayOptions, DisplayStyle}, utils::enums::HostStatus };

pub struct Cli;

impl Frontend for Cli {
    fn draw(display_data: &DisplayData) {
        let mut table = Builder::default().set_header(display_data.table_headers.clone());

        for (_, host_data) in display_data.hosts.iter() {

            let host_status = convert_to_string(&DataPoint::new(host_data.status.to_string()),
                                                &DisplayOptions::just_style(DisplayStyle::StatusUpDown));

            let mut row: Vec<String> = vec![ host_status,
                                             host_data.name.clone(),
                                             host_data.domain_name.clone(),
                                             host_data.ip_address.to_string() ];

            for monitor_id in &display_data.all_monitor_names {
                match host_data.monitoring_data.get(monitor_id) {
                    // There should always be some monitoring data if the key exists.
                    Some(monitoring_data) => row.push(convert_to_string(monitoring_data.values.last().unwrap(),
                                                                        &monitoring_data.display_options)),
                    None => row.push(String::from(""))
                }
            }

            table = table.add_row(row);
        }

        
        print!("{}", table.build().with(Style::psql()));
    }

}

#[derive(Tabled)]
struct TableEntry<'a> {
    pub name: &'a String,
    pub fqdn: &'a String,
    pub ip_address: &'a IpAddr,
    pub status: String,
}

fn convert_to_string(data_point: &DataPoint, display_options: &DisplayOptions) -> String {
    if data_point.is_empty() {
        if data_point.criticality == Criticality::Critical {
            "Error".red().to_string()
        }
        else {
            String::from("")
        }
    }
    else if display_options.use_multivalue {
        let mut separator = ", ";

        // Process all values and join them into string in the end.
        data_point.multivalue.iter().map(|data_point| {
            match display_options.display_style {
                DisplayStyle::CriticalityLevel => {
                    separator = "";

                    match data_point.criticality {
                        Criticality::NoData => "No data".to_string(),
                        Criticality::Normal => "▩".green().to_string(),
                        Criticality::Warning =>"▩".yellow().to_string(),
                        Criticality::Error => "▩".red().to_string(),
                        Criticality::Critical =>"▩".red().to_string(),
                    }
                },
                DisplayStyle::Numeric => {
                    String::from("TODO")
                },
                DisplayStyle::StatusUpDown => {
                    match HostStatus::from_str(&data_point.value).unwrap_or_default() {
                        HostStatus::Up => "Up".green().to_string(),
                        HostStatus::Down => "Down".red().to_string(),
                    }
                },
                DisplayStyle::String => {
                    data_point.value.to_string()
                },
            }
        }).collect::<Vec<String>>()
          .join(separator)
    }
    else {
        match display_options.display_style {
            DisplayStyle::CriticalityLevel => {
                match data_point.criticality {
                    Criticality::NoData => String::from("No data"),
                    Criticality::Normal => String::from("Normal"),
                    Criticality::Warning => String::from("Warning"),
                    Criticality::Error => String::from("Error"),
                    Criticality::Critical => String::from("Critical"),
                }
            },
            DisplayStyle::Numeric => {
                String::from("TODO")
            },
            DisplayStyle::StatusUpDown => {
                match HostStatus::from_str(&data_point.value).unwrap_or_default() {
                    HostStatus::Up => "Up".green().to_string(),
                    HostStatus::Down => "Down".red().to_string(),
                }
            },
            DisplayStyle::String => {
                data_point.value.to_string()
            },
        }
    }
}
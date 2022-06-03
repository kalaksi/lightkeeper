
use owo_colors::OwoColorize;
use std::net::IpAddr;
use tabled::{ Tabled, Table, Modify, Format, Style, object::Columns };
use super::{ Frontend, DisplayData };
use crate::{module::monitoring::Criticality, configuration::Host, utils::enums::HostStatus };

pub struct Cli;

impl Frontend for Cli {
    fn draw(display_data: &DisplayData) {
        let mut table = Vec::new();

        for (_, data) in display_data.hosts.iter() {
            let status_text = match &data.status {
                HostStatus::Up => "Up".green().to_string(),
                HostStatus::Down => "Down".red().to_string(),
            };

            table.push(TableEntry {
                name: &data.name,
                status: status_text,
                domain_name: &data.domain_name,
                ip_address: &data.ip_address,
            });
        }

        let table = Table::new(&table)
                        .with(Style::psql());
        print!("{}", table);
    }

}

#[derive(Tabled)]
struct TableEntry<'a> {
    pub name: &'a String,
    pub domain_name: &'a String,
    pub ip_address: &'a IpAddr,
    pub status: String,
}
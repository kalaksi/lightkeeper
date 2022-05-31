
use owo_colors::OwoColorize;
use std::net::IpAddr;
use tabled::{ Tabled, Table, Modify, Format, Style, object::Columns };

use super::{ Frontend, DisplayData };

pub struct Cli;

impl Frontend for Cli {
    fn draw(display_data: &DisplayData) {
        let mut table = Vec::new();

        for (_, data) in display_data.hosts.iter() {
            table.push(TableEntry {
                name: &data.name,
                status: data.status.to_string(),
                domain_name: &data.domain_name,
                ip_address: &data.ip_address,
            });
        }

        let table = Table::new(&table)
                        .with(Style::psql())   
                        .with(Modify::new(Columns::single(0)).with(Format::new(|s| s.red().to_string())));
        print!("{}", table);
    }

}

#[derive(Tabled)]
struct TableEntry<'a> {
    pub name: &'a String,
    pub domain_name: &'a String,
    pub status: String,
    pub ip_address: &'a IpAddr,
}
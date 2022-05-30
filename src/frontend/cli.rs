
use std::net::IpAddr;
use tabled::{ Tabled, Table, Style };
use super::{ Frontend, DisplayData };

pub struct Cli;

impl Frontend for Cli {
    fn draw(display_data: &DisplayData) {
        let mut table = Vec::new();
        let uptime = String::from("1");

        for (_, data) in display_data.hosts.iter() {
            table.push(TableEntry {
                name: &data.name,
                domain_name: &data.domain_name,
                ip_address: &data.ip_address,
                uptime: &uptime,
            });
        }

        let table = Table::new(table).to_string();
        print!("{}", table);
    }

}

#[derive(Tabled)]
struct TableEntry<'a> {
    pub name: &'a String,
    pub domain_name: &'a String,
    pub ip_address: &'a IpAddr,
    pub uptime: &'a String,
}
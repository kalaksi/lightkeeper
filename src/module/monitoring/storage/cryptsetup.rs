use serde_derive::Deserialize;
use serde_json;

use std::collections::HashMap;
use crate::enums::Criticality;
use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;


const KB_BYTES: u64 = 1000;
const MB_BYTES: u64 = 1000 * KB_BYTES;
const GB_BYTES: u64 = 1000 * MB_BYTES;
const TB_BYTES: u64 = 1000 * GB_BYTES;

#[monitoring_module(
    name="storage-cryptsetup",
    version="0.0.1",
    description="Gets information about cryptsetup (i.e. encrypted) devices.",
    settings={
        only_crypttab => "Only read /etc/crypttab instead. This allos finer control of presented devices.",
    }
)]
pub struct Cryptsetup {
    only_crypttab: bool,
}

impl Module for Cryptsetup {
    fn new(settings: &HashMap<String, String>) -> Cryptsetup {
        Cryptsetup {
            only_crypttab: settings.get("only_crypttab").unwrap_or(&String::from("false")).parse().unwrap(),
        }
    }
}

impl MonitoringModule for Cryptsetup {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Cryptsetup devices"),
            category: String::from("storage"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {

            if self.only_crypttab {
                Ok(String::from("grep -v '^#' /etc/crypttab"))
            }
            else {
                Ok(String::from("lsblk -b -p -o NAME,SIZE,FSTYPE,MOUNTPOINT --json"))
            }
        }
        else {
            Err(LkError::new_unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.message.is_empty() && response.return_code == 0 {
            return Ok(DataPoint::empty());
        }

        let mut result = DataPoint::empty();

        if self.only_crypttab {
            result.multivalue = response.message.lines().map(|line| {
                let mut parts = line.split_whitespace();
                let name = parts.next().unwrap().to_string();
                let device = parts.next().unwrap().to_string();
                let _keyfile = parts.next().unwrap().to_string();
                let _options = parts.next().unwrap().to_string();

                DataPoint::labeled_value_with_level(name, device, Criticality::Normal)
            }).collect();
        }
        else {
            let lsblk: Lsblk = serde_json::from_str(&response.message)
                .map_err(|e| format!("Failed to parse output: {}", e))?;

            result.multivalue = lsblk.blockdevices.iter()
                .filter(|block_device| block_device.fstype == Some(String::from("crypto_LUKS")))
                .map(|block_device| {
                    let pv_path = block_device.children.as_deref().unwrap_or_default().iter()
                        .filter(|child| child.fstype == Some(String::from("LVM2_member")))
                        .map(|child| child.name.clone())
                        .collect::<Vec<String>>();

                    let tags = if let Some(mountpoint) = &block_device.mountpoint {
                        vec![mountpoint.clone()]
                    }
                    else if !pv_path.is_empty() {
                        pv_path
                    }
                    else {
                        Vec::new()
                    };

                    // Change to SI units.
                    let size_bytes = block_device.size;
                    // match doesn't yet support exclusive ranges so this is simpler.
                    let si_size = if size_bytes < KB_BYTES {
                        format!("{} B", size_bytes)
                    }
                    else if size_bytes < MB_BYTES {
                        format!("{:.2} K", size_bytes as f64 / KB_BYTES as f64)
                    }
                    else if size_bytes < GB_BYTES {
                        format!("{:.2} M", size_bytes as f64 / MB_BYTES as f64)
                    }
                    else if size_bytes < TB_BYTES {
                        format!("{:.2} G", size_bytes as f64 / GB_BYTES as f64)
                    }
                    else {
                        format!("{:.2} T", size_bytes as f64 / TB_BYTES as f64)
                    };

                    let short_name = block_device.name.split('/').last().unwrap_or(&block_device.name);
                    DataPoint::labeled_value_with_level(short_name.to_string(), String::from(""), Criticality::Normal)
                              .with_tags(tags)
                              .with_description(format!("{}", si_size))

                })
                .collect();
        }

        Ok(result)
    }
}

#[derive(Deserialize)]
pub struct Lsblk {
    pub blockdevices: Vec<BlockDevice>,
}

#[derive(Deserialize)]
pub struct BlockDevice {
    pub name: String,
    pub size: u64,
    pub fstype: Option<String>,
    pub mountpoint: Option<String>,
    pub children: Option<Vec<BlockDevice>>,
}

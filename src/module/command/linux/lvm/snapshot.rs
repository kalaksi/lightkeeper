use chrono;

use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;


#[command_module("linux-lvm-snapshot", "0.0.1")]
pub struct Snapshot {
    pub snapshot_suffix: String,
    // Size in megabytes (unit M in LVM).
    pub snapshot_size_m: u32,
}

impl Module for Snapshot {
    fn new(settings: &HashMap<String, String>) -> Self {
        Snapshot {
            snapshot_suffix: settings.get("snapshot_suffix").unwrap_or(&String::from("_snapshot_TIME")).clone(),
            snapshot_size_m: settings.get("snapshot_size_m").unwrap_or(&String::from("2000")).parse::<u32>().unwrap(),
        }
    }
}

impl CommandModule for Snapshot {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("storage"),
            parent_id: String::from("linux-lvm-logical-volume"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("copy"),
            display_text: String::from("Create a snapshot"),
            depends_on_no_tags: vec![String::from("Snapshot")],
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> String {
        let lv_path = parameters.get(0).unwrap();
        let _vg_name = parameters.get(1).unwrap();
        let lv_name = parameters.get(2).unwrap();
        let _lv_size = parameters.get(3).unwrap();

        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let snapshot_suffix_with_timestamp = self.snapshot_suffix.replace("TIME", &timestamp);
        let snapshot_name = format!("{}{}", lv_name, snapshot_suffix_with_timestamp);
        let size_string = format!("{}M", self.snapshot_size_m);

        let mut command = ShellCommand::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            if host.platform.version_is_newer_than(platform_info::Flavor::Debian, "8") &&
               host.platform.version_is_older_than(platform_info::Flavor::Debian, "11") {
                 command.arguments(vec![
                      "lvcreate", "--snapshot", "--name", &snapshot_name, "--size", &size_string, lv_path
                 ]);
            };

            command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);
        }

        command.to_string()
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::new(String::new()))
        }
        else {
            Ok(CommandResult::new_error(response.message.clone()))
        }
    }
}
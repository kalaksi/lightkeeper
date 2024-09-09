use std::collections::HashMap;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;


#[command_module(
    name="storage-file-space-usage",
    version="0.0.1",
    description="Shows which files take the most disk space.",
    settings={
      line_count => "Number of lines to show. Default: 50.",
      one_file_system => "only show files on the same file system. Default: false."
    }
)]
pub struct FileSpaceUsage {
    pub line_count: u32,
    pub one_file_system: bool,
}

impl Module for FileSpaceUsage {
    fn new(settings: &HashMap<String, String>) -> Self {
        FileSpaceUsage {
            line_count: settings.get("line_count").unwrap_or(&String::from("50")).parse::<u32>().unwrap_or(100),
            one_file_system: settings.get("one_file_system").unwrap_or(&String::from("false")).parse::<bool>().unwrap_or(false),
        }
    }
}

impl CommandModule for FileSpaceUsage {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("storage"),
            parent_id: String::from("filesystem"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("search"),
            display_text: String::from("Find biggest files"),
            confirmation_text: String::from("Are you sure? This can take a while."),
            action: UIAction::TextDialog,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let mountpoint = &parameters[0];

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec!["du", "-x", "--block-size=1MB", mountpoint])
                   .pipe_to(vec!["sort", "-rn"])
                   .pipe_to(vec!["head", "-n", self.line_count.to_string().as_str()]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        let mut result_rows = Vec::new();
        let lines = response.message.lines();

        if response.is_error() {
            return Err(format!("Got exit code {} and {} lines of data", response.return_code, response.message));
        }
        else if lines.clone().count() == 0 {
            return Err(String::from("Successful execution but no data returned"))
        }

        for line in lines {
            let mut parts = line.split_whitespace();
            let size = parts.next().unwrap().parse::<u64>().unwrap();
            let path = parts.next().unwrap();

            // Trailing space is so that newline gets added properly since this is markdown.
            result_rows.push(format!("{} MB\t{}  ", size, path));
        }

        Ok(CommandResult::new_hidden(result_rows.join("\n")))
    }
}
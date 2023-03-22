use std::collections::VecDeque;
use std::process;


/// For building command line commands correctly.
pub struct ShellCommand {
    arguments: VecDeque<String>,
    pub use_sudo: bool,
}

// TODO: Use similar Into-approach elsewhere too.
impl ShellCommand {
    pub fn new() -> ShellCommand {
        ShellCommand {
            arguments: VecDeque::new(),
            use_sudo: false,
        }
    }

    pub fn argument<IntoString>(&mut self, argument: IntoString) -> &mut Self where IntoString: Into<String> {
        self.arguments.push_back(argument.into());
        self
    }

    pub fn arguments<IntoString>(&mut self, arguments: Vec<IntoString>) where IntoString: Into<String> {
        for argument in arguments {
            self.arguments.push_back(argument.into());
        }
    }
}

impl ToString for ShellCommand {
    fn to_string(&self) -> String {
        if self.arguments.is_empty() {
            String::new()
        }
        else {
            let mut command;
            if self.use_sudo {
                command = process::Command::new("sudo");
                for argument in self.arguments.iter() {
                    command.arg(argument);
                }
            }
            else {
                command = process::Command::new(&self.arguments[0]);
                for argument in self.arguments.iter().skip(1) {
                    command.arg(argument);
                }
            };

            format!("{:?}", command)
        }
    }
}
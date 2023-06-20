use std::collections::VecDeque;
use std::process;


/// For building command line commands correctly.
pub struct ShellCommand {
    arguments: VecDeque<String>,
    piped_to: VecDeque<Vec<String>>,
    pub use_sudo: bool,
}

impl ShellCommand {
    pub fn new() -> ShellCommand {
        ShellCommand {
            arguments: VecDeque::new(),
            piped_to: VecDeque::new(),
            use_sudo: false,
        }
    }

    pub fn argument<IntoString>(&mut self, argument: IntoString) -> &mut Self where IntoString: Into<String> {
        self.arguments.push_back(argument.into());
        self
    }

    pub fn arguments<IntoString>(&mut self, arguments: Vec<IntoString>) -> &mut Self where IntoString: Into<String> {
        for argument in arguments {
            self.arguments.push_back(argument.into());
        }
        self
    }

    pub fn pipe_to<IntoString>(&mut self, arguments: Vec<IntoString>) -> &mut Self where IntoString: Into<String> {
        let piped_with = arguments.into_iter().map(|argument| argument.into()).collect::<Vec<String>>();
        self.piped_to.push_back(piped_with);
        self
    }

    pub fn execute(&self) {
        if self.arguments.is_empty() {
            return;
        }

        let command = self.arguments.get(0).unwrap();
        let arguments = self.arguments.iter().skip(1).collect::<Vec<&String>>();

        if arguments.is_empty() {
            process::Command::new(command).output().expect("Running command failed");
        }
        else {
            process::Command::new(command).args(arguments).output().expect("Running command failed");
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

            let mut command_string = format!("{:?}", command);

            if !self.piped_to.is_empty() {
                for piped_arguments in self.piped_to.iter() {
                    let mut piped_command = process::Command::new(&piped_arguments[0]);
                    for argument in piped_arguments.iter().skip(1) {
                        piped_command.arg(argument);
                    }

                    command_string = format!("{} | {:?}", command_string, piped_command);
                }
            }

            command_string
        }
    }
}
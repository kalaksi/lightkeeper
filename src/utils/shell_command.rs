/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::VecDeque;
use std::process;

/// For building command line commands correctly.
pub struct ShellCommand {
    arguments: VecDeque<String>,
    piped_to: VecDeque<Vec<String>>,
    pub ignore_stderr: bool,
    pub use_sudo: bool,
}

impl ShellCommand {
    pub fn new() -> ShellCommand {
        ShellCommand {
            arguments: VecDeque::new(),
            piped_to: VecDeque::new(),
            ignore_stderr: false,
            use_sudo: false,
        }
    }

    pub fn new_from<IntoString>(arguments: Vec<IntoString>) -> ShellCommand
    where
        IntoString: Into<String>,
    {
        let mut new_command = Self::new();
        new_command.arguments(arguments);
        new_command
    }

    pub fn argument<IntoString>(&mut self, argument: IntoString) -> &mut Self
    where
        IntoString: Into<String>,
    {
        self.arguments.push_back(argument.into());
        self
    }

    pub fn arguments<IntoString>(&mut self, arguments: Vec<IntoString>) -> &mut Self
    where
        IntoString: Into<String>,
    {
        for argument in arguments {
            self.arguments.push_back(argument.into());
        }
        self
    }

    pub fn use_sudo(mut self) -> Self {
        self.use_sudo = true;
        self
    }

    pub fn pipe_to<IntoString>(&mut self, arguments: Vec<IntoString>) -> &mut Self
    where
        IntoString: Into<String>,
    {
        let piped_with = arguments.into_iter().map(|argument| argument.into()).collect::<Vec<String>>();
        self.piped_to.push_back(piped_with);
        self
    }

    pub fn execute(&self) -> std::io::Result<process::Output> {
        if self.arguments.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "No command specified"));
        }

        let command = self.arguments.get(0).unwrap();
        let arguments = self.arguments.iter().skip(1).collect::<Vec<&String>>();

        if arguments.is_empty() {
            process::Command::new(command).output()
        }
        else {
            process::Command::new(command).args(arguments).output()
        }
    }

    pub fn spawn(&self) -> std::io::Result<process::Child> {
        if self.arguments.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "No command specified"));
        }

        let command = self.arguments.get(0).unwrap();
        let arguments = self.arguments.iter().skip(1).collect::<Vec<&String>>();

        if arguments.is_empty() {
            process::Command::new(command).spawn()
        }
        else {
            process::Command::new(command).args(arguments).spawn()
        }
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.arguments.iter().cloned().collect::<_>()
    }
}

/// Wraps a string in single quotes with proper escaping for use in shell commands.
pub fn sh_single_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        }
        else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

impl ToString for ShellCommand {
    fn to_string(&self) -> String {
        if self.arguments.is_empty() {
            return String::new();
        }

        // Every argument is single-quoted so the shell never expands anything inside them.
        let quote_all = |arguments: &mut dyn Iterator<Item = &String>| {
            arguments.map(|argument| sh_single_quoted(argument)).collect::<Vec<_>>().join(" ")
        };

        let mut command_string = quote_all(&mut self.arguments.iter());
        if self.use_sudo {
            command_string = format!("'sudo' {}", command_string);
        }

        if self.ignore_stderr {
            command_string = format!("{} 2>/dev/null", command_string);
        }

        for piped_arguments in self.piped_to.iter() {
            command_string = format!("{} | {}", command_string, quote_all(&mut piped_arguments.iter()));
        }

        command_string
    }
}

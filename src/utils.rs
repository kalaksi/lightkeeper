pub mod string_manipulation;
pub use string_manipulation::strip_newline;
pub use string_manipulation::remove_whitespace;

pub mod version_number;
pub use version_number::VersionNumber;

pub mod string_validation;

pub mod shell_command;
pub use shell_command::ShellCommand;
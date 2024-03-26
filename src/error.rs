use strum_macros::Display;

use std::error;
use std::fmt;


#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new<Stringable: ToString>(kind: ErrorKind, message: Stringable) -> Error {
        Error {
            kind: kind,
            message: message.to_string(),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl error::Error for Error {
}

#[derive(Default, Debug, Display)]
pub enum ErrorKind {
    /// Connection timed out or was refused.
    ConnectionFailed,
    /// Encountered an unknown host key.
    HostKeyNotVerified,
    #[default]
    /// Other unspecified error.
    Other,
}
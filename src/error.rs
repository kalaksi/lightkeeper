use strum_macros::Display;

use std::error;
use std::fmt;


#[derive(Clone, Debug)]
pub struct LkError {
    kind: ErrorKind,
    message: String,
}

impl LkError {
    pub fn new<Stringable: ToString>(kind: ErrorKind, message: Stringable) -> LkError {
        LkError {
            kind: kind,
            message: message.to_string(),
        }
    }

    pub fn new_other<Stringable: ToString>(message: Stringable) -> LkError {
        LkError::new(ErrorKind::Other, message)
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

impl fmt::Display for LkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl error::Error for LkError {
}

impl From<std::io::Error> for LkError {
    fn from(error: std::io::Error) -> Self {
        LkError::new(ErrorKind::Other, error)
    }
}

impl From<String> for LkError {
    fn from(error: String) -> Self {
        LkError::new(ErrorKind::Other, error)
    }
}

#[derive(Clone, Default, Debug, Display)]
pub enum ErrorKind {
    /// Connection timed out, was refused or disconnected.
    ConnectionFailed,
    /// Encountered an unknown host key.
    HostKeyNotVerified,
    #[default]
    /// Other unspecified error.
    Other,
}
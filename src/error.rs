use strum_macros::Display;

use std::error;
use std::fmt;


#[derive(Clone, Debug)]
pub struct LkError {
    source: String,
    kind: ErrorKind,
    message: String,
}

impl LkError {
    pub fn new<Stringable: ToString>(kind: ErrorKind, message: Stringable) -> LkError {
        LkError {
            source: String::new(),
            kind: kind,
            message: message.to_string(),
        }
    }

    pub fn new_other<Stringable: ToString>(message: Stringable) -> LkError {
        LkError::new(ErrorKind::Other, message)
    }

    pub fn set_source<Stringable: ToString>(mut self, source: Stringable) -> LkError {
        self.source = source.to_string();
        self
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
        if self.source.is_empty() {
            write!(f, "{}", self.message)
        }
        else {
            write!(f, "{}: {}", self.source, self.message)
        }
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
    /// The requested operation is not supported on the platform.
    UnsupportedPlatform,
    /// Connection timed out, was refused or disconnected.
    ConnectionFailed,
    /// Encountered an unknown host key.
    HostKeyNotVerified,
    #[default]
    /// Other unspecified error.
    Other,
}
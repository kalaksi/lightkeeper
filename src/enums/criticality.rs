use serde_derive::{Serialize, Deserialize};
use strum_macros::Display;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize, Display)]
pub enum Criticality {
    Ignore,
    Normal,
    // Info is basically Normal level but it will be displayed to user in some cases where Normal won't.
    Info,
    NoData,
    Warning,
    Error,
    Critical,
}
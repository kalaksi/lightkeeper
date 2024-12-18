use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Serialize, Deserialize, Display, EnumString)]
pub enum Criticality {
    Ignore,
    Normal,
    /// Info is basically Normal level but it will be displayed to user in some cases where Normal won't.
    Info,
    /// Currently same as "unknown" or "pending". Initial result.
    NoData,
    Warning,
    Error,
    Critical,
}

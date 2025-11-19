/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Serialize, Deserialize, Display, EnumString)]
pub enum Criticality {
    Ignore,
    Normal,
    /// Info is basically Normal level but it will be displayed to user in some cases where Normal won't.
    Info,
    /// Currently same as "unknown" or "pending". Initial result. Default.
    #[default]
    NoData,
    Warning,
    Error,
    Critical,
    /// When command or service is not available in the system and therefore can't be monitored.
    NotAvailable,
}


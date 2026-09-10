/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[repr(u8)]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Serialize, Deserialize, Display, EnumString)]
pub enum Criticality {
    /// When command or service is not available in the system and therefore can't be monitored.
    NotAvailable,
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
}

impl TryFrom<u8> for Criticality {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            0 => Ok(Self::NotAvailable),
            1 => Ok(Self::Ignore),
            2 => Ok(Self::Normal),
            3 => Ok(Self::Info),
            4 => Ok(Self::NoData),
            5 => Ok(Self::Warning),
            6 => Ok(Self::Error),
            7 => Ok(Self::Critical),
            _ => Err(()),
        }
    }
}

/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Preferred transport for reaching lightkeeper-core on an admin host.
#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoreTransportPreference {
    #[default]
    OpensshStreamLocal,
}

impl FromStr for CoreTransportPreference {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openssh_stream_local" => Ok(CoreTransportPreference::OpensshStreamLocal),
            _ => Err(()),
        }
    }
}

impl Display for CoreTransportPreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreTransportPreference::OpensshStreamLocal => write!(f, "openssh_stream_local"),
        }
    }
}

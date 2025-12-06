/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::default::Default;
use std::fmt::Display;
use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Default, Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HostStatus {
    Unknown,
    #[default]
    Pending,
    Up,
    Down,
}

impl FromStr for HostStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match String::from(s).to_lowercase().as_str() {
            "unknown" => Ok(HostStatus::Unknown),
            "pending" => Ok(HostStatus::Pending),
            "up" => Ok(HostStatus::Up),
            "down" => Ok(HostStatus::Down),
            _ => panic!("Invalid HostStatus '{}'", s),
        }
    }
}

impl Display for HostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostStatus::Unknown => write!(f, "unknown"),
            HostStatus::Pending => write!(f, "pending"),
            HostStatus::Up => write!(f, "up"),
            HostStatus::Down => write!(f, "down"),
        }
    }
}

/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fmt::Display;
use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EditMode {
    #[default]
    Regular,
    Vim,
    Emacs,
}

impl FromStr for EditMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "regular" => Ok(EditMode::Regular),
            "vim" => Ok(EditMode::Vim),
            "emacs" => Ok(EditMode::Emacs),
            _ => Err(()),
        }
    }
}

impl Display for EditMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditMode::Regular => write!(f, "regular"),
            EditMode::Vim => write!(f, "vim"),
            EditMode::Emacs => write!(f, "emacs"),
        }
    }
}

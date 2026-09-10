/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pub mod string_manipulation;
pub use string_manipulation::*;

pub mod version_number;
pub use version_number::VersionNumber;

pub mod string_validation;

pub mod shell_command;
pub use shell_command::{sh_single_quoted, ShellCommand};

pub mod error_message;
pub use error_message::ErrorMessage;

pub mod sha256;

pub mod journalctl_time;
pub use journalctl_time::is_valid_journalctl_time;

pub mod journalctl_format;
pub use journalctl_format::journalctl_json_to_rich_text;

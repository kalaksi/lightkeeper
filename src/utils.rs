/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

pub mod string_manipulation;
pub use string_manipulation::*;

pub mod version_number;
pub use version_number::VersionNumber;

pub mod string_validation;

pub mod host_address;

pub mod shell_command;
pub use shell_command::{sh_single_quoted, ShellCommand};

pub mod error_message;
pub use error_message::ErrorMessage;

pub mod sha256;

pub mod journalctl_time;
pub use journalctl_time::is_valid_journalctl_time;

pub mod journalctl_format;
pub use journalctl_format::journalctl_json_to_rich_text;

/// Disables core dumps and same-user ptrace so that unlocked secrets can't be read from process memory.
pub fn disable_process_dumping() -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    if unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

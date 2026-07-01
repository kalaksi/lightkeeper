/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use crate::error::LkError;
use super::download;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::{UIAction, *};
use crate::utils::sh_single_quoted;
use lightkeeper_module::command_module;

#[command_module(
    name="_internal-filebrowser-move",
    version="0.0.1",
    description="Move files or directories on the remote host.",
    uses_sudo=true,
)]
pub struct FileBrowserMove {
}

impl Module for FileBrowserMove {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserMove {
        }
    }
}

impl CommandModule for FileBrowserMove {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Hidden,
            display_icon: String::from("edit-cut"),
            display_text: String::from("Move files"),
            parent_id: String::from("_internal-filebrowser-ls"),
            action: UIAction::FollowOutput,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        if host.platform.os != platform_info::OperatingSystem::Linux {
            return Err(LkError::unsupported_platform());
        }

        if parameters.len() < 2 {
            return Err(LkError::other("Destination and at least one source path required"));
        }
        let destination = parameters.first().ok_or(LkError::other("No destination specified"))?;
        let sources = &parameters[1..];
        if destination.is_empty() {
            return Err(LkError::other("Destination is empty"));
        }
        for source in sources {
            if source.is_empty() {
                return Err(LkError::other("Empty source path"));
            }
        }

        let sudo = if host.settings.contains(&HostSetting::UseSudo) { "sudo " } else { "" };
        let dest = match destination.ends_with('/') {
            true => destination.clone(),
            false => format!("{}/", destination),
        };

        let quoted_sources = sources.iter()
            .map(|source| sh_single_quoted(source))
            .collect::<Vec<_>>()
            .join(" ");

        // Detect-and-branch per source: when the source is on the same filesystem as the
        // destination, mv is an instant atomic rename, so use it. When they're on different
        // filesystems, mv would silently copy without progress, so use rsync instead.
        //
        // Identifies the filesystem with `stat -c %d`, equal ids mean the same filesystem.
        let out_format = sh_single_quoted(download::RSYNC_OUT_FORMAT);
        let script = format!(
            concat!(
                "dst={dest} && ",
                "dst_dev=$({sudo}stat -c %d -- \"$dst\") && ",
                "for src in {quoted_sources}; do ",
                    "src_dev=$({sudo}stat -c %d -- \"$src\") || exit 1; ",
                    "if [ \"$src_dev\" = \"$dst_dev\" ]; then ",
                        "{sudo}mv -n -- \"$src\" \"$dst\" || exit $?; ",
                    "else ",
                        // `--` stops option parsing so a path starting with '-' can never be
                        // treated as an rsync flag. Paths are absolute, so the ':' in any name
                        // always follows a '/' and is never mistaken for a host:path remote spec.
                        "{sudo}env LANG=C LC_ALL=C rsync -av --info=progress2 --stats ",
                        "--ignore-existing --remove-source-files {out_format} -- \"$src\" \"$dst\" || exit $?; ",
                        // --remove-source-files only removes files, leaving empty source
                        // directories behind, so clean those up to complete the move.
                        "{sudo}find -- \"$src\" -depth -type d -empty -delete 2>/dev/null || true; ",
                    "fi; ",
                "done"
            ),
            dest = sh_single_quoted(&dest),
            sudo = sudo,
            quoted_sources = quoted_sources,
            out_format = out_format,
        );

        Ok(script)
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            Ok(download::process_rsync_partial_response(response, "Moving"))
        }
        else {
            Ok(download::process_rsync_final_response(response, "Move"))
        }
    }
}

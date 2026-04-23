/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

mod api;
mod local;
mod remote;

use std::path::PathBuf;

pub use api::{CommandBackend, LocalBackendApi};
pub use local::LocalCommandBackend;
pub use remote::RemoteCommandBackend;

use crate::command_handler::CommandHandler;
use crate::monitor_manager::MonitorManager;

pub fn new_local_command_backend(command_handler: CommandHandler, monitor_manager: MonitorManager) -> Box<dyn CommandBackend> {
    Box::new(LocalCommandBackend::new(command_handler, monitor_manager))
}

pub fn new_remote_command_backend(socket_path: PathBuf) -> Box<dyn CommandBackend> {
    Box::new(RemoteCommandBackend::new(socket_path))
}

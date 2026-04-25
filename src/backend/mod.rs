/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

mod api;
mod local;
mod local_config;
mod remote;
mod remote_config;

pub use api::{CommandBackend, ConfigBackend, LocalBackendApi};
pub use local::LocalCommandBackend;
pub use local_config::LocalConfigBackend;
pub use remote::{RemoteCommandBackend, RemoteCoreClient};
pub use remote_config::RemoteConfigBackend;

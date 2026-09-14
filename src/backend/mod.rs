/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

mod api;
mod core_connection;
mod local;
mod local_config;
mod remote;
mod remote_config;
pub mod ssh_auth;
pub mod ssh_transport;

pub use api::{CommandBackend, ConfigBackend, LocalBackendApi};
pub use core_connection::{CoreConnectionState, CoreConnectionStatus};
pub use local::LocalCommandBackend;
pub use local_config::LocalConfigBackend;
pub use remote::{RemoteCommandBackend, RemoteCoreClient};
pub use remote_config::RemoteConfigBackend;
pub use ssh_auth::{HostKeyChallenge, SshAuthError};
pub use ssh_transport::{
    accept_remote_core_host_key, probe_remote_core_host, RemoteCoreHostProbe, CoreInstallPlan,
};

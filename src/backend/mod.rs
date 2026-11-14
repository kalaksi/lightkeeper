/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

mod api;
mod core_connection;
mod docker_path_maps;
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
pub use remote::{build_desktop_rsync_command, RemoteCommandBackend, RemoteCoreClient};
pub use remote_config::RemoteConfigBackend;
pub use ssh_auth::{HostKeyChallenge, SshAuthError};
pub use ssh_transport::{
    accept_remote_core_host_key,
    install_remote_core,
    probe_remote_core_host,
    CoreInstallPlan,
    RemoteCoreHostProbe,
};

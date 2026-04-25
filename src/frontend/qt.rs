/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


mod models;

pub mod qml_frontend;
pub use models::{
    CommandBackend,
    ConfigBackend,
    LocalConfigBackend,
    RemoteConfigBackend,
    RemoteCoreClient,
};
pub use qml_frontend::QmlFrontend;
mod resources;
mod resources_qml;
/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::sync::Arc;

use super::remote::RemoteCoreClient;

pub struct RemoteConfigBackend {
    pub(crate) client: Arc<RemoteCoreClient>,
}

impl RemoteConfigBackend {
    pub fn new(client: Arc<RemoteCoreClient>) -> Self {
        RemoteConfigBackend { client }
    }
}

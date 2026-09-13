/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::sync::Arc;

use lightkeeper::configuration::{Configuration, Hosts};
use lightkeeper::remote_core::runtime::CoreRuntime;
use lightkeeper::ModuleFactory;

use super::MemorySecretStore;

/// Core runtime backed by an in-memory secret store (no system keyring).
pub fn test_core_runtime(main_config: &Configuration, hosts: &Hosts, config_dir: String) -> CoreRuntime {
    CoreRuntime::new_with(
        main_config,
        hosts,
        Arc::new(ModuleFactory::new()),
        config_dir,
        Arc::new(MemorySecretStore::new()),
    )
    .unwrap()
}

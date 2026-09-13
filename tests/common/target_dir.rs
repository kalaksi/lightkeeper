/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Creates a unique directory under the crate `target/` dir. Left for cargo clean.
pub fn unique_target_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join(format!(
        "lk-{}-{}-{}",
        prefix,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Restores a process environment variable when dropped. Does not delete any paths.
pub struct EnvVarGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvVarGuard {
    pub fn set(key: &'static str, value: impl AsRef<OsStr>) -> Self {
        let previous = env::var_os(key);
        // SAFETY: tests run single-threaded per harness invocation for this override lifetime.
        unsafe {
            env::set_var(key, value);
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        // SAFETY: paired with set(); restores prior process state for this key only.
        unsafe {
            match &self.previous {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }
}

/// Points `XDG_CACHE_HOME` at a unique `target/` dir so cache writes stay out of `$HOME`.
pub fn redirect_xdg_cache_home() -> EnvVarGuard {
    let dir = unique_target_dir("xdg-cache");
    EnvVarGuard::set("XDG_CACHE_HOME", &dir)
}

/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::path::PathBuf;
use std::sync::Arc;

use qmetaobject;

#[cfg(all(debug_assertions, feature = "hot-reload"))]
use std::{thread, time::Instant};

#[cfg(all(debug_assertions, feature = "hot-reload"))]
pub fn watch(path: PathBuf, engine: Arc<qmetaobject::QmlEngine>) {
    use notify::{self, Watcher};

    thread::spawn(move || {
        let (notify_sender, notify_receiver) = std::sync::mpsc::channel();
        let mut watcher = match notify::RecommendedWatcher::new(notify_sender, notify::Config::default()) {
            Ok(watcher) => watcher,
            Err(error) => {
                log::error!("Hot reload failed to initialize: {}", error);
                return;
            }
        };

        if let Err(error) = watcher.watch(&path, notify::RecursiveMode::Recursive) {
            log::error!("Hot reload failed to initialize: {}", error);
            return;
        }

        log::info!("Hot reload is in use");

        let mut last_reload = Instant::now();

        loop {
            if let Ok(event_result) = notify_receiver.recv() {
                let event = match event_result {
                    Ok(event) => event,
                    Err(_error) => {
                        log::error!("Hot reload stopped");
                        break;
                    }
                };

                if event.kind.is_access() || last_reload.elapsed().as_millis() < 500 {
                    continue;
                }

                log::debug!("Reload triggered by file {}", event.paths[0].display());
                engine.trim_component_cache();
                engine.clear_component_cache();
                last_reload = Instant::now();
            }
        }
    });
}

#[cfg(not(all(debug_assertions, feature = "hot-reload")))]
pub fn watch(_path: PathBuf, _engine: Arc<qmetaobject::QmlEngine>) {
}

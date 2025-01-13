use std::{
    path::PathBuf,
    thread,
    time::Instant
};
use qmetaobject;
use std::sync::Arc;

#[cfg(debug_assertions)]
pub fn watch(path: PathBuf, engine: Arc<qmetaobject::QmlEngine>) {

    use notify::{self, Watcher};

    thread::spawn(move || {
        let (notify_sender, notify_receiver) = std::sync::mpsc::channel();
        let mut watcher = notify::RecommendedWatcher::new(notify_sender, notify::Config::default()).unwrap();

        if let Err(error) = watcher.watch(&path, notify::RecursiveMode::Recursive) {
            log::error!("Hot reload failed to initialize: {:?}", error);
            return;
        }

        log::info!("Hot reload is in use");

        let mut last_reload = Instant::now();

        loop {
            let event_result = notify_receiver.recv().unwrap();
            if let Ok(event) = event_result {
                if event.kind.is_access() || last_reload.elapsed().as_millis() < 500 {
                    continue;
                }

                log::debug!("Reload triggered by file {:?}", event.paths[0].display());
                engine.trim_component_cache();
                engine.clear_component_cache();
                last_reload = Instant::now();
            }
        }
    });
}

#[cfg(not(debug_assertions))]
pub fn watch(_path: PathBuf, _engine: Arc<qmetaobject::QmlEngine>) {
    // No hot reload in release mode.
}

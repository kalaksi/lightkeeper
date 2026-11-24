/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::configuration::{NewsConfig, NewsFeed};
use crate::frontend::UIUpdate;
use crate::news::parse_feed;

use std::sync::mpsc;

const MAX_ITEMS: usize = 30;

#[derive(Clone, Debug, Serialize)]
pub struct NewsItem {
    pub id: String,
    pub title: String,
    pub link: String,
    pub published: Option<DateTime<Utc>>,
    pub source: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary: String,
}

pub struct NewsManager {
    config: Arc<Mutex<NewsConfig>>,
    items: Arc<Mutex<Vec<NewsItem>>>,
    stop: Arc<AtomicBool>,
    wake_sender: Option<mpsc::Sender<()>>,
    thread: Option<JoinHandle<()>>,
    update_sender: mpsc::Sender<UIUpdate>,
}

impl NewsManager {
    pub fn new(config: NewsConfig, update_sender: mpsc::Sender<UIUpdate>) -> Self {
        let mut manager = Self {
            config: Arc::new(Mutex::new(config)),
            items: Arc::new(Mutex::new(Vec::new())),
            stop: Arc::new(AtomicBool::new(false)),
            wake_sender: None,
            thread: None,
            update_sender,
        };
        manager.start_polling();
        manager
    }

    pub fn latest_title(&self) -> Option<String> {
        self.items.lock().ok()?.first().map(|item| item.title.clone())
    }

    pub fn items(&self) -> Vec<NewsItem> {
        self.items.lock().map(|guard| guard.clone()).unwrap_or_default()
    }

    pub fn config(&self) -> NewsConfig {
        self.config.lock().map(|guard| guard.clone()).unwrap_or_default()
    }

    pub fn set_config(&self, config: NewsConfig) {
        let enabled = config.enabled;
        if let Ok(mut guard) = self.config.lock() {
            *guard = config;
        }
        if enabled {
            self.refresh_now();
        }
        else {
            if let Ok(mut guard) = self.items.lock() {
                guard.clear();
            }
            let _ = self.update_sender.send(UIUpdate::NewsUpdated);
        }
    }

    pub fn refresh_now(&self) {
        if let Some(sender) = &self.wake_sender {
            let _ = sender.send(());
        }
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(sender) = self.wake_sender.take() {
            let _ = sender.send(());
        }
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }

    fn start_polling(&mut self) {
        let (wake_sender, wake_receiver) = mpsc::channel();
        self.wake_sender = Some(wake_sender);

        let config = Arc::clone(&self.config);
        let items = Arc::clone(&self.items);
        let stop = Arc::clone(&self.stop);
        let update_sender = self.update_sender.clone();

        self.thread = thread::Builder::new()
            .name(String::from("news-poller"))
            .spawn(move || {
                // Updates queue on the UI channel until receiveUpdates starts reading.
                if !stop.load(Ordering::SeqCst) {
                    Self::poll_once(&config, &items, &update_sender);
                }

                loop {
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }

                    let wait_secs = config
                        .lock()
                        .map(|guard| {
                            if guard.enabled && guard.refresh_interval > 0 {
                                guard.refresh_interval.max(60)
                            }
                            else {
                                3600
                            }
                        })
                        .unwrap_or(3600);

                    let woken = match wake_receiver.recv_timeout(Duration::from_secs(wait_secs)) {
                        Ok(()) => true,
                        Err(mpsc::RecvTimeoutError::Timeout) => false,
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    };
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }

                    let should_poll = woken
                        || config
                            .lock()
                            .map(|guard| guard.enabled && guard.refresh_interval > 0)
                            .unwrap_or(false);
                    if should_poll {
                        Self::poll_once(&config, &items, &update_sender);
                    }
                }
            })
            .ok();
    }

    fn poll_once(
        config: &Arc<Mutex<NewsConfig>>,
        items: &Arc<Mutex<Vec<NewsItem>>>,
        update_sender: &mpsc::Sender<UIUpdate>,
    ) {
        let snapshot = match config.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => return,
        };

        let enabled_feeds: Vec<NewsFeed> = if snapshot.enabled {
            snapshot.feeds.into_iter().filter(|feed| feed.enabled).collect()
        }
        else {
            Vec::new()
        };
        if enabled_feeds.is_empty() {
            if let Ok(mut guard) = items.lock() {
                guard.clear();
            }
            let _ = update_sender.send(UIUpdate::NewsUpdated);
            return;
        }

        let mut merged: HashMap<String, NewsItem> = HashMap::new();
        let mut any_success = false;

        let agent = ureq::Agent::new_with_defaults();
        for feed in enabled_feeds {
            match agent.get(&feed.url).call() {
                Ok(mut response) => {
                    match response.body_mut().read_to_string() {
                        Ok(body) => {
                            any_success = true;
                            for item in parse_feed(&body, &feed.name) {
                                merged.insert(item.id.clone(), item);
                            }
                        }
                        Err(error) => {
                            log::warn!("Failed to read news feed {}: {}", feed.name, error);
                        }
                    }
                }
                Err(error) => {
                    log::warn!("Failed to fetch news feed {}: {}", feed.name, error);
                }
            }
        }

        // Keep last good items when every feed fails.
        if !any_success {
            return;
        }

        let mut list: Vec<NewsItem> = merged.into_values().collect();
        list.sort_by(|left, right| {
            right.published.cmp(&left.published)
                .then_with(|| left.title.cmp(&right.title))
        });
        list.truncate(MAX_ITEMS);

        if let Ok(mut guard) = items.lock() {
            *guard = list;
        }
        let _ = update_sender.send(UIUpdate::NewsUpdated);
    }
}

impl Drop for NewsManager {
    fn drop(&mut self) {
        self.stop();
    }
}

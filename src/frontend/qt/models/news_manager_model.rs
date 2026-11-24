/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

extern crate qmetaobject;

use qmetaobject::*;
use serde_json;

use crate::configuration::NewsConfig;
use crate::news::NewsManager;

#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct NewsManagerModel {
    base: qt_base_class!(trait QObject),

    latestTitle: qt_property!(QString; READ get_latest_title NOTIFY newsChanged),

    getItemsJson: qt_method!(fn(&self) -> QString),
    refreshNow: qt_method!(fn(&self)),

    newsChanged: qt_signal!(),

    news_manager: Option<NewsManager>,
}

#[allow(non_snake_case)]
impl NewsManagerModel {
    pub fn new(news_manager: Option<NewsManager>) -> Self {
        Self {
            news_manager,
            ..Default::default()
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut manager) = self.news_manager.take() {
            manager.stop();
        }
    }

    pub fn notify_updated(&mut self) {
        self.newsChanged();
    }

    fn get_latest_title(&self) -> QString {
        let title = self.news_manager
            .as_ref()
            .and_then(|manager| manager.latest_title())
            .unwrap_or_default();
        QString::from(title)
    }

    fn getItemsJson(&self) -> QString {
        let items = self.news_manager.as_ref().map(|manager| manager.items()).unwrap_or_default();
        QString::from(serde_json::to_string(&items).unwrap_or_else(|_| String::from("[]")))
    }

    fn refreshNow(&self) {
        if let Some(manager) = self.news_manager.as_ref() {
            manager.refresh_now();
        }
    }

    pub fn replace_config_from_main(&self, config: NewsConfig) {
        if let Some(manager) = self.news_manager.as_ref() {
            manager.set_config(config);
        }
    }
}

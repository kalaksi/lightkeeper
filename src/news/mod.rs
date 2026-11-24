/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

mod news_manager;
mod parse;

pub use news_manager::{NewsItem, NewsManager};
pub use parse::parse_feed;

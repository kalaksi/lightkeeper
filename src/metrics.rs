/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pub mod lmserver;

#[cfg(feature = "gui")]
pub mod metric;
#[cfg(feature = "gui")]
pub use metric::*;

#[cfg(feature = "gui")]
pub mod metrics_manager;
#[cfg(feature = "gui")]
pub use metrics_manager::MetricsManager;

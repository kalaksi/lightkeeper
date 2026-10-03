/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pub mod lmserver;

#[cfg(any(feature = "gui", feature = "core"))]
pub mod metric;
#[cfg(any(feature = "gui", feature = "core"))]
pub use metric::*;

#[cfg(any(feature = "gui", feature = "core"))]
pub mod metrics_manager;
#[cfg(any(feature = "gui", feature = "core"))]
pub use metrics_manager::MetricsManager;

/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Metric {
    pub time: i64,
    pub label: String,
    pub value: f32,
}

impl From<Metric> for crate::metrics::lmserver::Metric {
    fn from(metric: Metric) -> Self {
        crate::metrics::lmserver::Metric {
            time: metric.time,
            label: metric.label,
            value: metric.value,
        }
    }
}

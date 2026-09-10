/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

///
/// This module contains the client-server communication protocol of the locally run LMServer metrics server.
/// Protocol version 1.1
///
use std::collections::HashMap;
#[cfg(feature = "gui")]
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Metric {
    pub time: i64,
    pub label: String,
    pub value: f32,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct AlertEvent {
    /// Unix timestamp in seconds.
    pub time: i64,
    pub host_id: String,
    pub monitor_id: String,
    pub entry_id: String,
    pub label: String,
    pub value: String,
    /// `Criticality` as `u8` (`Criticality as u8` / `Criticality::try_from`).
    pub from_level: u8,
    /// `Criticality` as `u8` (`Criticality as u8` / `Criticality::try_from`).
    pub to_level: u8,
}

#[cfg(feature = "gui")]
#[derive(Serialize, Deserialize)]
pub struct LMSRequest {
    pub request_id: u64,
    /// Requester sets this to unix time in milliseconds.
    pub time: u32,
    pub request_type: RequestType,
}

#[cfg(feature = "gui")]
#[derive(Serialize, Deserialize)]
pub enum RequestType {
    Healthcheck,
    Exit,
    MetricsInsert {
        host_id: String,
        metric_id: String,
        metrics: Vec<Metric>,
    },
    MetricsQuery {
        host_id: String,
        metric_id: String,
        /// Unix timestamp in seconds.
        start_time: i64,
        /// Unix timestamp in seconds.
        end_time: i64,
    },
    AlertInsert {
        events: Vec<AlertEvent>,
    },
    /// Empty `host_id` or `monitor_id` means no filter on that field.
    AlertQuery {
        host_id: String,
        monitor_id: String,
        /// Unix timestamp in seconds.
        start_time: i64,
        /// Unix timestamp in seconds.
        end_time: i64,
    },
}

#[cfg(feature = "gui")]
impl LMSRequest {
    pub fn exit() -> Self {
        LMSRequest {
            request_id: 0,
            time: 0,
            request_type: RequestType::Exit,
        }
    }
}

#[cfg(feature = "gui")]
impl Debug for LMSRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServiceRequest({})", self.request_id)
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LMSResponse {
    pub request_id: u64,
    /// In milliseconds. 0 if not set.
    pub lag: u32,
    pub metrics: HashMap<String, Vec<Metric>>,
    pub alerts: Vec<AlertEvent>,
    pub errors: Vec<String>,
}

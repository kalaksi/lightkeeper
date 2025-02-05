///
/// This module contains the client-server communication protocol of the locally run TMServer metrics server.
/// Protocol version 1.0
///
use std::collections::HashMap;
use std::fmt::Debug;

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Metric {
    pub time: i64,
    pub label: String,
    pub value: i64,
}

#[derive(Serialize, Deserialize)]
pub struct TMSRequest {
    pub request_id: u64,
    /// Requester sets this to unix time in milliseconds.
    pub time: u32,
    pub request_type: RequestType,
}

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
}

impl TMSRequest {
    pub fn exit() -> Self {
        TMSRequest {
            request_id: 0,
            time: 0,
            request_type: RequestType::Exit,
        }
    }
}

impl Debug for TMSRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServiceRequest({})", self.request_id)
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct TMSResponse {
    pub request_id: u64,
    /// In milliseconds. 0 if not set.
    pub lag: u32,
    pub metrics: HashMap<String, Vec<Metric>>,
    pub errors: Vec<String>,
}

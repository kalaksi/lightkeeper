use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Metric {
    pub time: i64,
    pub label: String,
    pub value: i64,
}

impl From<Metric> for crate::metrics::tmserver::Metric {
    fn from(metric: Metric) -> Self {
        crate::metrics::tmserver::Metric {
            time: metric.time,
            label: metric.label,
            value: metric.value,
        }
    }
}

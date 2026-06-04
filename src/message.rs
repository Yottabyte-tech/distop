use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkerInfo {
    pub hostname: String,
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub load_average_1min: f64,
    pub process_count: usize,
    pub status: String,
}

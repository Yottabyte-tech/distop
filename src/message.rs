use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::get_data::processes::ProcessInfo;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkerInfo {
    pub hostname: String,
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub load_average_1min: f64,
    pub process_count: usize,
    pub status: String,
    pub processes: Vec<ProcessInfo>,
    pub cores: Vec<f32>,
}

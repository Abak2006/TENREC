// TENREC - Process & Application Domain Models
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessItem {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub executable_path: String,
    pub publisher: String,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub memory_percent: f64,
    pub net_upload_bps: u64,
    pub net_download_bps: u64,
    pub connection_count: usize,
    pub start_time: String,
    pub is_elevated: bool,
    pub application_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationItem {
    pub id: String,
    pub name: String,
    pub publisher: String,
    pub executable_path: String,
    pub process_count: usize,
    pub pids: Vec<u32>,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub net_upload_bps: u64,
    pub net_download_bps: u64,
    pub connection_count: usize,
    pub first_seen: String,
    pub last_seen: String,
    pub baseline_status: String,
    pub security_status: String,
}

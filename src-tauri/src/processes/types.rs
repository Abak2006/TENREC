// TENREC - Process & Application Types
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessItem {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    /// Empty when Windows denies access to the image path (protected/system processes).
    pub executable_path: String,
    /// CompanyName from the file's version resource. Not signature-verified.
    pub company: Option<String>,
    pub application_id: String,
    pub application_name: String,
    /// Share of total CPU capacity (0-100), normalised by logical core count.
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub memory_percent: f64,
    /// Process I/O (all file, device and network I/O counted by Windows), bytes/sec.
    pub io_read_bps: u64,
    pub io_write_bps: u64,
    pub connection_count: usize,
    /// RFC 3339 start time, empty when unknown.
    pub start_time: String,
    pub runs_from_user_writable_location: bool,
    /// Windows denied query access (service/system process while TENREC is not elevated):
    /// CPU and I/O read as 0 but are actually unknown.
    pub restricted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationItem {
    pub id: String,
    pub name: String,
    pub company: Option<String>,
    pub executable_path: String,
    pub process_count: usize,
    pub pids: Vec<u32>,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub io_read_bps: u64,
    pub io_write_bps: u64,
    pub connection_count: usize,
    /// Number of this application's processes whose CPU/I/O could not be read.
    pub restricted_process_count: usize,
    /// First time TENREC saw this application during the current session.
    pub first_seen: String,
    pub baseline_status: String, // "learning" | "normal" | "divergent"
}

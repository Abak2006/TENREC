// TENREC - Network Domain Models
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub id: String,
    pub pid: u32,
    pub process_name: String,
    pub application_name: String,
    pub protocol: String, // "TCP" | "UDP"
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: String,
    pub hostname: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub first_seen: String,
    pub last_seen: String,
    pub is_unusual: bool,
    pub is_new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListeningPortInfo {
    pub port: u16,
    pub protocol: String,
    pub pid: u32,
    pub process_name: String,
    pub application_name: String,
    pub bind_address: String,
    pub state: String,
    pub is_known_service: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopDestination {
    pub destination: String,
    pub hostname: String,
    pub application: String,
    pub total_bytes: u64,
    pub connection_count: usize,
    pub last_seen: String,
    pub security_status: String, // "normal" | "unusual" | "suspicious"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkThroughput {
    pub bytes_in_sec: u64,
    pub bytes_out_sec: u64,
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub latency_ms: Option<f64>,
    pub packet_loss_percent: Option<f64>,
    pub active_connection_count: usize,
    pub primary_interface: String,
    pub link_speed_mbps: Option<u64>,
}

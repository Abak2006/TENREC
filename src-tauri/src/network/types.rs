// TENREC - Network Types
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub id: String,
    pub pid: u32,
    pub process_name: String,
    pub application_name: String,
    pub protocol: String,   // "TCP"
    pub ip_version: u8,     // 4 | 6
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    /// Name from the local DNS cache, when Windows resolved this address recently.
    pub hostname: Option<String>,
    pub state: String,
    pub is_public_remote: bool,
    /// True when this remote address was first seen in the current sample.
    pub is_new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListeningPortInfo {
    pub port: u16,
    pub protocol: String, // "TCP" | "UDP"
    pub ip_version: u8,
    pub pid: u32,
    pub process_name: String,
    pub application_name: String,
    pub bind_address: String,
    /// Bound to 0.0.0.0 / :: (reachable from other machines unless the firewall blocks it).
    pub all_interfaces: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopDestination {
    pub destination: String,
    pub hostname: Option<String>,
    /// Applications that currently or previously connected to this address.
    pub applications: Vec<String>,
    /// Connections open to this address right now.
    pub open_connections: usize,
    /// Number of samples in which a connection to this address was observed.
    pub times_seen: u64,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkThroughput {
    pub bytes_in_sec: u64,
    pub bytes_out_sec: u64,
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub active_connection_count: usize,
    /// Interface with the most traffic in the last interval, if any had traffic.
    pub busiest_interface: Option<String>,
}

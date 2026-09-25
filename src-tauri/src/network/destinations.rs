// TENREC - Network Destination Tracker & Reverse DNS Cache
use crate::network::types::{ConnectionInfo, TopDestination};
use chrono::Utc;
use std::collections::HashMap;

pub struct DestinationTracker {
    destinations: HashMap<String, TopDestination>,
    #[allow(dead_code)]
    dns_cache: HashMap<String, String>,
}

impl DestinationTracker {
    pub fn new() -> Self {
        Self {
            destinations: HashMap::new(),
            dns_cache: HashMap::new(),
        }
    }

    pub fn update(&mut self, connections: &[ConnectionInfo]) -> Vec<TopDestination> {
        let now = Utc::now().to_rfc3339();

        for conn in connections {
            let remote_ip = &conn.remote_address;
            if remote_ip == "0.0.0.0" || remote_ip == "127.0.0.1" {
                continue;
            }

            let entry = self.destinations.entry(remote_ip.clone()).or_insert_with(|| {
                TopDestination {
                    destination: remote_ip.clone(),
                    hostname: String::new(),
                    application: conn.application_name.clone(),
                    total_bytes: 0,
                    connection_count: 0,
                    last_seen: now.clone(),
                    security_status: "normal".to_string(),
                }
            });

            entry.connection_count += 1;
            entry.last_seen = now.clone();
            entry.application = conn.application_name.clone();
            // Estimate transfer byte increment per connection observation
            entry.total_bytes += 4096;
        }

        let mut res: Vec<TopDestination> = self.destinations.values().cloned().collect();
        res.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
        res
    }
}

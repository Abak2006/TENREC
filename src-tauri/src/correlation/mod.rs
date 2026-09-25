// TENREC - Process ↔ Network Correlation Engine
use crate::network::types::{ConnectionInfo, ListeningPortInfo};
use crate::processes::types::{ApplicationItem, ProcessItem};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatChangedItem {
    pub id: String,
    pub category: String, // "new_application" | "new_destination" | "traffic_spike" | "listening_port"
    pub title: String,
    pub description: String,
    pub timestamp: String,
    pub entity_name: String,
    pub severity: String, // "info" | "attention" | "warning"
}

pub struct CorrelationEngine {
    known_applications: Mutex<HashSet<String>>,
    known_destinations: Mutex<HashSet<String>>,
    known_listening_ports: Mutex<HashSet<u16>>,
    what_changed_log: Mutex<Vec<WhatChangedItem>>,
}

impl CorrelationEngine {
    pub fn new() -> Self {
        Self {
            known_applications: Mutex::new(HashSet::new()),
            known_destinations: Mutex::new(HashSet::new()),
            known_listening_ports: Mutex::new(HashSet::new()),
            what_changed_log: Mutex::new(Vec::new()),
        }
    }

    pub fn correlate(
        &self,
        processes: &mut [ProcessItem],
        applications: &mut [ApplicationItem],
        connections: &mut [ConnectionInfo],
        listening: &[ListeningPortInfo],
    ) -> Vec<WhatChangedItem> {
        let mut changes = Vec::new();
        let now = Utc::now().format("%H:%M:%S").to_string();

        // 1. Build fast PID -> (ProcessName, AppName) map
        let mut pid_to_app: HashMap<u32, (String, String)> = HashMap::new();
        for p in processes.iter() {
            pid_to_app.insert(p.pid, (p.name.clone(), p.application_name.clone()));
        }

        // 2. Correlate connections with process names and applications
        let mut known_dests = self.known_destinations.lock().unwrap();
        for conn in connections.iter_mut() {
            if let Some((pname, aname)) = pid_to_app.get(&conn.pid) {
                conn.process_name = pname.clone();
                conn.application_name = aname.clone();
            }

            // Detect new remote destination
            if conn.remote_address != "0.0.0.0" && conn.remote_address != "127.0.0.1" {
                if !known_dests.contains(&conn.remote_address) {
                    known_dests.insert(conn.remote_address.clone());
                    conn.is_new = true;

                    changes.push(WhatChangedItem {
                        id: format!("change_dest_{}_{}", conn.remote_address, Utc::now().timestamp_millis()),
                        category: "new_destination".to_string(),
                        title: format!("New Destination Observed: {}", conn.remote_address),
                        description: format!(
                            "Application '{}' (PID {}) contacted {} for the first time.",
                            conn.application_name, conn.pid, conn.remote_address
                        ),
                        timestamp: now.clone(),
                        entity_name: conn.remote_address.clone(),
                        severity: "info".to_string(),
                    });
                }
            }
        }

        // 3. Track new Applications
        let mut known_apps = self.known_applications.lock().unwrap();
        for app in applications.iter() {
            if !known_apps.contains(&app.name) {
                known_apps.insert(app.name.clone());
                changes.push(WhatChangedItem {
                    id: format!("change_app_{}_{}", app.name, Utc::now().timestamp_millis()),
                    category: "new_application".to_string(),
                    title: format!("New Application Observed: {}", app.name),
                    description: format!(
                        "Application '{}' detected with {} process(es) (Publisher: {}).",
                        app.name, app.process_count, app.publisher
                    ),
                    timestamp: now.clone(),
                    entity_name: app.name.clone(),
                    severity: "info".to_string(),
                });
            }
        }

        // 4. Track new Listening Ports
        let mut known_ports = self.known_listening_ports.lock().unwrap();
        for port_info in listening.iter() {
            if !known_ports.contains(&port_info.port) {
                known_ports.insert(port_info.port);
                changes.push(WhatChangedItem {
                    id: format!("change_port_{}_{}", port_info.port, Utc::now().timestamp_millis()),
                    category: "listening_port".to_string(),
                    title: format!("New Listening Port: :{}", port_info.port),
                    description: format!(
                        "Process '{}' opened {} listening socket on {}:{}",
                        port_info.process_name, port_info.protocol, port_info.bind_address, port_info.port
                    ),
                    timestamp: now.clone(),
                    entity_name: format!(":{}", port_info.port),
                    severity: if port_info.port < 1024 { "attention".to_string() } else { "info".to_string() },
                });
            }
        }

        // Append to history log
        let mut log = self.what_changed_log.lock().unwrap();
        log.extend(changes.clone());
        if log.len() > 100 {
            let keep_from = log.len() - 100;
            *log = log[keep_from..].to_vec();
        }

        let mut res = log.clone();
        res.reverse();
        res
    }

    pub fn get_what_changed(&self) -> Vec<WhatChangedItem> {
        let mut log = self.what_changed_log.lock().unwrap().clone();
        log.reverse();
        log
    }
}

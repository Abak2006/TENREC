// TENREC - "What changed" tracking
//
// The first sample only seeds the known sets: everything already running when TENREC
// starts is the baseline, not a change.
use crate::network::types::{ConnectionInfo, ListeningPortInfo};
use crate::processes::types::ApplicationItem;
use crate::util::LockExt;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;

const MAX_LOG: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatChangedItem {
    pub id: String,
    pub category: String, // "new_application" | "new_destination" | "listening_port"
    pub title: String,
    pub description: String,
    /// RFC 3339
    pub timestamp: String,
    pub entity_name: String,
    pub severity: String, // "info" | "attention"
}

#[derive(Default)]
struct Inner {
    seeded: bool,
    apps: HashSet<String>,
    destinations: HashSet<String>,
    ports: HashSet<(String, u16, u32)>,
    log: Vec<WhatChangedItem>,
    seq: u64,
}

#[derive(Default)]
pub struct CorrelationEngine {
    inner: Mutex<Inner>,
}

impl CorrelationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records changes and marks connections to never-before-seen public addresses as new.
    pub fn observe(
        &self,
        applications: &[ApplicationItem],
        connections: &mut [ConnectionInfo],
        listening: &[ListeningPortInfo],
    ) -> Vec<WhatChangedItem> {
        let mut g = self.inner.lock_safe();
        let now = Utc::now().to_rfc3339();
        let seeding = !g.seeded;
        let mut changes = Vec::new();

        for app in applications {
            if g.apps.insert(app.id.clone()) && !seeding {
                g.seq += 1;
                changes.push(WhatChangedItem {
                    id: format!("chg_{}", g.seq),
                    category: "new_application".to_string(),
                    title: format!("{} started", app.name),
                    description: format!(
                        "{} process(es){}",
                        app.process_count,
                        if app.executable_path.is_empty() { String::new() } else { format!(" from {}", app.executable_path) }
                    ),
                    timestamp: now.clone(),
                    entity_name: app.name.clone(),
                    severity: "info".to_string(),
                });
            }
        }

        for conn in connections.iter_mut().filter(|c| c.is_public_remote) {
            if g.destinations.insert(conn.remote_address.clone()) && !seeding {
                conn.is_new = true;
                g.seq += 1;
                changes.push(WhatChangedItem {
                    id: format!("chg_{}", g.seq),
                    category: "new_destination".to_string(),
                    title: format!("New destination {}", conn.remote_address),
                    description: format!("{} (PID {}) connected to {}:{}", conn.process_name, conn.pid, conn.remote_address, conn.remote_port),
                    timestamp: now.clone(),
                    entity_name: conn.remote_address.clone(),
                    severity: "info".to_string(),
                });
            }
        }

        for p in listening {
            let key = (p.protocol.clone(), p.port, p.pid);
            if g.ports.insert(key) && !seeding {
                g.seq += 1;
                changes.push(WhatChangedItem {
                    id: format!("chg_{}", g.seq),
                    category: "listening_port".to_string(),
                    title: format!("{} now listening on {} {}", p.process_name, p.protocol, p.port),
                    description: format!(
                        "Bound to {}{}",
                        p.bind_address,
                        if p.all_interfaces { " (all interfaces: reachable from the network unless firewalled)" } else { "" }
                    ),
                    timestamp: now.clone(),
                    entity_name: format!("{}:{}", p.protocol, p.port),
                    severity: if p.all_interfaces && p.protocol == "TCP" { "attention" } else { "info" }.to_string(),
                });
            }
        }

        g.seeded = true;
        g.log.extend(changes.iter().cloned());
        let len = g.log.len();
        if len > MAX_LOG {
            g.log.drain(..len - MAX_LOG);
        }
        changes
    }

    pub fn get_what_changed(&self) -> Vec<WhatChangedItem> {
        let g = self.inner.lock_safe();
        g.log.iter().rev().cloned().collect()
    }
}

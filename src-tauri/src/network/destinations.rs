// TENREC - Remote destination history (public addresses only)
use crate::network::types::{ConnectionInfo, TopDestination};
use chrono::Utc;
use std::collections::{HashMap, HashSet};

const MAX_TRACKED: usize = 2000;

#[derive(Default)]
pub struct DestinationTracker {
    destinations: HashMap<String, TopDestination>,
}

impl DestinationTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, connections: &[ConnectionInfo]) -> Vec<TopDestination> {
        let now = Utc::now().to_rfc3339();
        let mut open: HashMap<&str, usize> = HashMap::new();
        let mut apps: HashMap<&str, HashSet<&str>> = HashMap::new();
        let mut names: HashMap<&str, &str> = HashMap::new();
        for c in connections.iter().filter(|c| c.is_public_remote) {
            *open.entry(c.remote_address.as_str()).or_insert(0) += 1;
            apps.entry(c.remote_address.as_str()).or_default().insert(c.application_name.as_str());
            if let Some(h) = &c.hostname {
                names.insert(c.remote_address.as_str(), h.as_str());
            }
        }

        for d in self.destinations.values_mut() {
            d.open_connections = 0;
        }
        for (addr, count) in &open {
            let entry = self.destinations.entry(addr.to_string()).or_insert_with(|| TopDestination {
                destination: addr.to_string(),
                hostname: None,
                applications: Vec::new(),
                open_connections: 0,
                times_seen: 0,
                first_seen: now.clone(),
                last_seen: now.clone(),
            });
            entry.open_connections = *count;
            entry.times_seen += 1;
            entry.last_seen = now.clone();
            if let Some(h) = names.get(addr) {
                entry.hostname = Some(h.to_string());
            }
            for a in apps.get(addr).into_iter().flatten() {
                if !entry.applications.iter().any(|x| x == a) {
                    entry.applications.push(a.to_string());
                }
            }
        }

        if self.destinations.len() > MAX_TRACKED {
            // Drop the least recently seen entries.
            let mut keys: Vec<(String, String)> =
                self.destinations.values().map(|d| (d.last_seen.clone(), d.destination.clone())).collect();
            keys.sort();
            for (_, k) in keys.into_iter().take(self.destinations.len() - MAX_TRACKED) {
                self.destinations.remove(&k);
            }
        }

        let mut res: Vec<TopDestination> = self.destinations.values().cloned().collect();
        res.sort_by(|a, b| b.open_connections.cmp(&a.open_connections).then(b.times_seen.cmp(&a.times_seen)));
        res.truncate(200);
        res
    }
}

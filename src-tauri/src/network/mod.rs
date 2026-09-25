// TENREC - Network Subsystem
pub mod types;
pub mod sockets;
pub mod throughput;
pub mod destinations;

pub use destinations::DestinationTracker;
pub use sockets::{collect_active_tcp_connections, collect_active_udp_endpoints};
pub use throughput::ThroughputTracker;
pub use types::{ConnectionInfo, ListeningPortInfo, NetworkThroughput, TopDestination};

use std::collections::HashMap;
use std::sync::Mutex;

pub struct NetworkManager {
    throughput_tracker: Mutex<ThroughputTracker>,
    destination_tracker: Mutex<DestinationTracker>,
    cached_connections: Mutex<Vec<ConnectionInfo>>,
    cached_listening: Mutex<Vec<ListeningPortInfo>>,
    cached_destinations: Mutex<Vec<TopDestination>>,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            throughput_tracker: Mutex::new(ThroughputTracker::new()),
            destination_tracker: Mutex::new(DestinationTracker::new()),
            cached_connections: Mutex::new(Vec::new()),
            cached_listening: Mutex::new(Vec::new()),
            cached_destinations: Mutex::new(Vec::new()),
        }
    }

    pub fn refresh(&self, process_names: &HashMap<u32, (String, String)>) -> (Vec<ConnectionInfo>, Vec<ListeningPortInfo>) {
        let (conns, mut listening) = collect_active_tcp_connections(process_names);
        let udp_ports = collect_active_udp_endpoints(process_names);
        listening.extend(udp_ports);

        let dests = {
            let mut tracker = self.destination_tracker.lock().unwrap();
            tracker.update(&conns)
        };

        *self.cached_connections.lock().unwrap() = conns.clone();
        *self.cached_listening.lock().unwrap() = listening.clone();
        *self.cached_destinations.lock().unwrap() = dests;

        (conns, listening)
    }

    pub fn get_connections(&self) -> Vec<ConnectionInfo> {
        let lock = self.cached_connections.lock().unwrap();
        if lock.is_empty() {
            // First time call without prior refresh
            drop(lock);
            let empty_map = HashMap::new();
            let (conns, _) = self.refresh(&empty_map);
            conns
        } else {
            lock.clone()
        }
    }

    pub fn get_listening_ports(&self) -> Vec<ListeningPortInfo> {
        let lock = self.cached_listening.lock().unwrap();
        if lock.is_empty() {
            drop(lock);
            let empty_map = HashMap::new();
            let (_, listening) = self.refresh(&empty_map);
            listening
        } else {
            lock.clone()
        }
    }

    pub fn get_top_destinations(&self) -> Vec<TopDestination> {
        let lock = self.cached_destinations.lock().unwrap();
        if lock.is_empty() {
            drop(lock);
            let conns = self.get_connections();
            let mut tracker = self.destination_tracker.lock().unwrap();
            tracker.update(&conns)
        } else {
            lock.clone()
        }
    }

    pub fn get_throughput(&self) -> NetworkThroughput {
        let conns_count = self.cached_connections.lock().unwrap().len();
        let mut tracker = self.throughput_tracker.lock().unwrap();
        tracker.collect(conns_count)
    }
}

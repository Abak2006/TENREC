// TENREC - Interface throughput (all adapters, via sysinfo)
use crate::network::types::NetworkThroughput;
use std::time::Instant;
use sysinfo::Networks;

pub struct ThroughputTracker {
    networks: Networks,
    last: Instant,
}

impl Default for ThroughputTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ThroughputTracker {
    pub fn new() -> Self {
        let networks = Networks::new_with_refreshed_list();
        Self { networks, last: Instant::now() }
    }

    pub fn collect(&mut self, active_connection_count: usize) -> NetworkThroughput {
        // `true` also drops adapters that disappeared and picks up new ones.
        self.networks.refresh(true);
        let now = Instant::now();
        let secs = now.duration_since(self.last).as_secs_f64().max(0.001);
        self.last = now;

        let (mut d_in, mut d_out, mut t_in, mut t_out) = (0u64, 0u64, 0u64, 0u64);
        let mut busiest: Option<(String, u64)> = None;
        for (name, data) in &self.networks {
            let rx = data.received();
            let tx = data.transmitted();
            d_in += rx;
            d_out += tx;
            t_in += data.total_received();
            t_out += data.total_transmitted();
            if rx + tx > 0 && busiest.as_ref().map(|(_, b)| rx + tx > *b).unwrap_or(true) {
                busiest = Some((name.clone(), rx + tx));
            }
        }

        NetworkThroughput {
            bytes_in_sec: (d_in as f64 / secs) as u64,
            bytes_out_sec: (d_out as f64 / secs) as u64,
            total_bytes_in: t_in,
            total_bytes_out: t_out,
            active_connection_count,
            busiest_interface: busiest.map(|(n, _)| n),
        }
    }
}

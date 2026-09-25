// TENREC - Real Network Interface Throughput Tracker
use crate::network::types::NetworkThroughput;
use sysinfo::Networks;

pub struct ThroughputTracker {
    networks: Networks,
    prev_total_in: u64,
    prev_total_out: u64,
    has_prev: bool,
}

impl ThroughputTracker {
    pub fn new() -> Self {
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);
        Self {
            networks,
            prev_total_in: 0,
            prev_total_out: 0,
            has_prev: false,
        }
    }

    pub fn collect(&mut self, active_conns_count: usize) -> NetworkThroughput {
        self.networks.refresh(true);

        let mut total_in: u64 = 0;
        let mut total_out: u64 = 0;
        let mut delta_in: u64 = 0;
        let mut delta_out: u64 = 0;
        let mut primary_iface = "Ethernet / Wi-Fi".to_string();

        for (name, data) in &self.networks {
            let rx = data.received();
            let tx = data.transmitted();
            let tot_rx = data.total_received();
            let tot_tx = data.total_transmitted();

            delta_in += rx;
            delta_out += tx;
            total_in += tot_rx;
            total_out += tot_tx;

            if rx > 0 || tx > 0 {
                primary_iface = name.clone();
            }
        }

        self.prev_total_in = total_in;
        self.prev_total_out = total_out;
        self.has_prev = true;

        NetworkThroughput {
            bytes_in_sec: delta_in,
            bytes_out_sec: delta_out,
            total_bytes_in: total_in,
            total_bytes_out: total_out,
            latency_ms: None,
            packet_loss_percent: None,
            active_connection_count: active_conns_count,
            primary_interface: primary_iface,
            link_speed_mbps: Some(1000),
        }
    }
}

// TENREC - Behavioral Baseline Engine
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBaseline {
    pub application_name: String,
    pub sample_count: u64,
    pub avg_cpu_percent: f32,
    pub peak_cpu_percent: f32,
    pub avg_memory_bytes: u64,
    pub peak_memory_bytes: u64,
    pub avg_net_bps: f64,
    pub peak_net_bps: f64,
    pub status: String, // "learning" | "normal" | "divergent"
}

pub struct BaselineEngine {
    baselines: Mutex<HashMap<String, AppBaseline>>,
}

impl BaselineEngine {
    pub fn new() -> Self {
        Self {
            baselines: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_sample(&self, app_name: &str, cpu: f32, memory: u64, net_bps: f64) {
        let mut map = self.baselines.lock().unwrap();
        let baseline = map.entry(app_name.to_string()).or_insert_with(|| AppBaseline {
            application_name: app_name.to_string(),
            sample_count: 0,
            avg_cpu_percent: cpu,
            peak_cpu_percent: cpu,
            avg_memory_bytes: memory,
            peak_memory_bytes: memory,
            avg_net_bps: net_bps,
            peak_net_bps: net_bps,
            status: "learning".to_string(),
        });

        baseline.sample_count += 1;

        // Exponential moving average (alpha = 0.1 for smoothed baseline)
        let alpha = 0.1f32;
        baseline.avg_cpu_percent = (1.0 - alpha) * baseline.avg_cpu_percent + alpha * cpu;
        baseline.peak_cpu_percent = baseline.peak_cpu_percent.max(cpu);

        let alpha_f64 = 0.1f64;
        baseline.avg_memory_bytes = ((1.0 - alpha_f64) * (baseline.avg_memory_bytes as f64) + alpha_f64 * (memory as f64)) as u64;
        baseline.peak_memory_bytes = baseline.peak_memory_bytes.max(memory);

        baseline.avg_net_bps = (1.0 - alpha_f64) * baseline.avg_net_bps + alpha_f64 * net_bps;
        baseline.peak_net_bps = baseline.peak_net_bps.max(net_bps);

        // Status determination
        if baseline.sample_count < 10 {
            baseline.status = "learning".to_string();
        } else if cpu > (baseline.avg_cpu_percent * 3.5).max(40.0)
            || (net_bps > (baseline.avg_net_bps * 5.0).max(5_000_000.0))
        {
            baseline.status = "divergent".to_string();
        } else {
            baseline.status = "normal".to_string();
        }
    }

    pub fn get_baseline(&self, app_name: &str) -> Option<AppBaseline> {
        let map = self.baselines.lock().unwrap();
        map.get(app_name).cloned()
    }

    pub fn get_status(&self, app_name: &str) -> String {
        let map = self.baselines.lock().unwrap();
        map.get(app_name)
            .map(|b| b.status.clone())
            .unwrap_or_else(|| "learning".to_string())
    }
}

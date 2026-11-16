// TENREC - Background telemetry sampler
//
// One thread samples processes, sockets, throughput and system metrics together every
// SAMPLE_INTERVAL and publishes an immutable snapshot. IPC commands only read the latest
// snapshot, so they are cheap, consistent with each other, and never block the UI thread
// on a full process scan.
use crate::baseline::BaselineEngine;
use crate::correlation::CorrelationEngine;
use crate::network::{collect_sockets, connection_counts, ConnectionInfo, DestinationTracker, ListeningPortInfo, NetworkThroughput, ProcessNames, ThroughputTracker, TopDestination};
use crate::persistence::Database;
use crate::processes::{ApplicationItem, ProcessItem, ProcessManager};
use crate::security::SecurityEngine;
use crate::system::{IoMetrics, SystemManager, SystemSnapshot};
use crate::util::RwLockExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub const SAMPLE_INTERVAL: Duration = Duration::from_secs(2);
const PERSIST_INTERVAL: Duration = Duration::from_secs(30);
const RETENTION_HOURS: u32 = 24 * 7;

#[derive(Clone, Default)]
pub struct Sample {
    pub system: Option<SystemSnapshot>,
    pub throughput: Option<NetworkThroughput>,
    pub processes: Vec<ProcessItem>,
    pub applications: Vec<ApplicationItem>,
    pub connections: Vec<ConnectionInfo>,
    pub listening: Vec<ListeningPortInfo>,
    pub destinations: Vec<TopDestination>,
    pub sequence: u64,
    pub last_error: Option<String>,
}

pub struct Collectors {
    processes: ProcessManager,
    system: SystemManager,
    throughput: ThroughputTracker,
    destinations: DestinationTracker,
}

impl Default for Collectors {
    fn default() -> Self {
        Self::new()
    }
}

impl Collectors {
    pub fn new() -> Self {
        Self {
            processes: ProcessManager::new(),
            system: SystemManager::new(),
            throughput: ThroughputTracker::new(),
            destinations: DestinationTracker::new(),
        }
    }
}

pub struct TelemetryHub {
    latest: RwLock<Arc<Sample>>,
    pub correlation: CorrelationEngine,
    pub baseline: BaselineEngine,
    pub security: SecurityEngine,
    running: AtomicBool,
}

impl Default for TelemetryHub {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryHub {
    pub fn new() -> Self {
        Self {
            latest: RwLock::new(Arc::new(Sample::default())),
            correlation: CorrelationEngine::new(),
            baseline: BaselineEngine::new(),
            security: SecurityEngine::new(),
            running: AtomicBool::new(false),
        }
    }

    pub fn latest(&self) -> Arc<Sample> {
        self.latest.read_safe().clone()
    }

    /// Runs one full sampling cycle and publishes the result.
    pub fn tick(&self, c: &mut Collectors) -> Arc<Sample> {
        let mut processes = c.processes.sample();
        let names: ProcessNames = processes.iter().map(|p| (p.pid, (p.name.clone(), p.application_name.clone()))).collect();
        let (mut connections, listening) = collect_sockets(&names);
        let counts = connection_counts(&connections);
        for p in processes.iter_mut() {
            p.connection_count = counts.get(&p.pid).copied().unwrap_or(0);
        }
        let mut applications = c.processes.group_applications(&processes);

        self.correlation.observe(&applications, &mut connections, &listening);
        for app in applications.iter_mut() {
            self.baseline.record_sample(&app.id, app.cpu_percent as f32, app.memory_bytes, (app.io_read_bps + app.io_write_bps) as f64);
            app.baseline_status = self.baseline.get_status(&app.id);
        }
        self.security.analyze(&processes, &connections, &listening);

        let io = IoMetrics {
            read_bytes_sec: processes.iter().map(|p| p.io_read_bps).sum(),
            write_bytes_sec: processes.iter().map(|p| p.io_write_bps).sum(),
        };
        let system = c.system.snapshot(io);
        let throughput = c.throughput.collect(connections.len());
        let destinations = c.destinations.update(&connections);

        let prev_seq = self.latest.read_safe().sequence;
        let sample = Arc::new(Sample {
            system: Some(system),
            throughput: Some(throughput),
            processes,
            applications,
            connections,
            listening,
            destinations,
            sequence: prev_seq + 1,
            last_error: None,
        });
        *self.latest.write_safe() = sample.clone();
        sample
    }

    /// Starts the sampling thread once. Panics inside a cycle are caught and reported
    /// through `last_error` so one bad sample cannot stop telemetry.
    /// `after_sample` runs on the sampler thread after each successful sample (Focus mode).
    pub fn start(self: &Arc<Self>, db: Arc<Database>, after_sample: impl Fn(&Sample) + Send + 'static) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        let hub = self.clone();
        std::thread::Builder::new()
            .name("tenrec-sampler".into())
            .spawn(move || {
                if let Err(e) = db.purge_old_telemetry(RETENTION_HOURS) {
                    log::warn!("Telemetry purge failed: {e}");
                }
                let mut collectors = Collectors::new();
                let mut last_persist: Option<Instant> = None;
                loop {
                    let started = Instant::now();
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| hub.tick(&mut collectors)));
                    match result {
                        Ok(sample) => {
                            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| after_sample(&sample))).is_err() {
                                log::error!("Post-sample handler panicked; continuing");
                            }
                            if last_persist.map(|t| t.elapsed() >= PERSIST_INTERVAL).unwrap_or(true) {
                                last_persist = Some(Instant::now());
                                if let (Some(s), Some(t)) = (&sample.system, &sample.throughput) {
                                    if let Err(e) = db.insert_system_sample(
                                        &s.timestamp,
                                        s.cpu.usage_percent as f32,
                                        s.memory.used_bytes,
                                        s.memory.total_bytes,
                                        s.io.read_bytes_sec as f64,
                                        s.io.write_bytes_sec as f64,
                                        t.bytes_in_sec as f64,
                                        t.bytes_out_sec as f64,
                                        s.power.battery_percent.map(|p| p as f32),
                                        s.power.ac_connected,
                                    ) {
                                        log::warn!("Could not store telemetry sample: {e}");
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            log::error!("Telemetry sampling cycle panicked; continuing");
                            let mut s = (*hub.latest()).clone();
                            s.last_error = Some("A sampling cycle failed. Data may be briefly out of date.".into());
                            *hub.latest.write_safe() = Arc::new(s);
                            collectors = Collectors::new();
                        }
                    }
                    std::thread::sleep(SAMPLE_INTERVAL.saturating_sub(started.elapsed()));
                }
            })
            .expect("failed to spawn telemetry thread");
    }
}

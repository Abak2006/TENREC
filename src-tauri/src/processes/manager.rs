// TENREC - Process & Application Inventory
use crate::processes::types::{ApplicationItem, ProcessItem};
use crate::processes::version_info::{read_version_info, VersionInfo};
use crate::util::is_user_writable_location;
use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use sysinfo::{ProcessesToUpdate, System};

pub struct ProcessManager {
    sys: System,
    version_cache: HashMap<String, VersionInfo>,
    app_first_seen: HashMap<String, String>,
    last_refresh: Option<Instant>,
    logical_cores: usize,
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let logical_cores = sys.cpus().len().max(1);
        Self {
            sys,
            version_cache: HashMap::new(),
            app_first_seen: HashMap::new(),
            last_refresh: Some(Instant::now()),
            logical_cores,
        }
    }

    fn version_for(&mut self, path: &str) -> VersionInfo {
        if path.is_empty() {
            return VersionInfo::default();
        }
        let key = path.to_ascii_lowercase();
        if let Some(v) = self.version_cache.get(&key) {
            return v.clone();
        }
        let v = read_version_info(path);
        if self.version_cache.len() > 4096 {
            self.version_cache.clear();
        }
        self.version_cache.insert(key, v.clone());
        v
    }

    fn app_identity(name: &str, path: &str, info: &VersionInfo) -> (String, String) {
        let id_source = if path.is_empty() { name.to_ascii_lowercase() } else { path.to_ascii_lowercase() };
        let id = format!("app_{:x}", fxhash(&id_source));
        let stem = Path::new(name).file_stem().and_then(|s| s.to_str()).unwrap_or(name).to_string();
        let display = info
            .description
            .clone()
            .or_else(|| info.product.clone())
            .unwrap_or(stem);
        (id, display)
    }

    /// Refresh the process table once and build the process list.
    /// `connection_count` is left at 0; the telemetry sampler fills it in.
    pub fn sample(&mut self) -> Vec<ProcessItem> {
        self.sys.refresh_memory();
        self.sys.refresh_processes(ProcessesToUpdate::All, true);
        let now = Instant::now();
        let elapsed = self
            .last_refresh
            .map(|t| now.duration_since(t).as_secs_f64())
            .unwrap_or(0.0)
            .max(0.001);
        self.last_refresh = Some(now);

        let total_mem = self.sys.total_memory() as f64;
        let cores = self.logical_cores as f64;

        let raw: Vec<_> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, p)| {
                let disk = p.disk_usage();
                (
                    pid.as_u32(),
                    p.parent().map(|pp| pp.as_u32()),
                    p.name().to_string_lossy().to_string(),
                    p.exe().map(|e| e.to_string_lossy().to_string()).unwrap_or_default(),
                    p.cpu_usage() as f64,
                    p.memory(),
                    disk.read_bytes,
                    disk.written_bytes,
                    p.start_time(),
                )
            })
            .collect();

        let mut items = Vec::with_capacity(raw.len());
        for (pid, ppid, name, path, cpu, mem, read, written, start) in raw {
            let info = self.version_for(&path);
            let (app_id, app_name) = Self::app_identity(&name, &path, &info);
            let start_time = if start > 0 {
                Utc.timestamp_opt(start as i64, 0).single().map(|t| t.to_rfc3339()).unwrap_or_default()
            } else {
                String::new()
            };
            items.push(ProcessItem {
                pid,
                ppid,
                runs_from_user_writable_location: is_user_writable_location(&path),
                // sysinfo reads the image path and CPU times with the same access right,
                // so a missing path means CPU/I-O were not readable either.
                restricted: path.is_empty() && pid > 4,
                name,
                executable_path: path,
                company: info.company.clone(),
                application_id: app_id,
                application_name: app_name,
                cpu_percent: (cpu / cores).clamp(0.0, 100.0),
                memory_bytes: mem,
                memory_percent: if total_mem > 0.0 { (mem as f64 / total_mem * 100.0).clamp(0.0, 100.0) } else { 0.0 },
                io_read_bps: (read as f64 / elapsed) as u64,
                io_write_bps: (written as f64 / elapsed) as u64,
                connection_count: 0, // filled in by the sampler once sockets are collected
                start_time,
            });
        }
        items.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent));
        items
    }

    pub fn group_applications(&mut self, processes: &[ProcessItem]) -> Vec<ApplicationItem> {
        let mut groups: HashMap<&str, Vec<&ProcessItem>> = HashMap::new();
        for p in processes {
            // PID 0/4 (Idle/System) carry no useful application identity.
            if p.pid == 0 {
                continue;
            }
            groups.entry(p.application_id.as_str()).or_default().push(p);
        }

        let now = Utc::now().to_rfc3339();
        let mut apps: Vec<ApplicationItem> = groups
            .into_iter()
            .map(|(id, procs)| {
                let first = procs[0];
                let first_seen = self.app_first_seen.entry(id.to_string()).or_insert_with(|| now.clone()).clone();
                ApplicationItem {
                    id: id.to_string(),
                    name: first.application_name.clone(),
                    company: first.company.clone(),
                    executable_path: first.executable_path.clone(),
                    process_count: procs.len(),
                    pids: procs.iter().map(|p| p.pid).collect(),
                    cpu_percent: procs.iter().map(|p| p.cpu_percent).sum::<f64>().min(100.0),
                    memory_bytes: procs.iter().map(|p| p.memory_bytes).sum(),
                    io_read_bps: procs.iter().map(|p| p.io_read_bps).sum(),
                    io_write_bps: procs.iter().map(|p| p.io_write_bps).sum(),
                    connection_count: procs.iter().map(|p| p.connection_count).sum(),
                    restricted_process_count: procs.iter().filter(|p| p.restricted).count(),
                    first_seen,
                    baseline_status: "learning".to_string(),
                }
            })
            .collect();
        apps.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent).then(b.memory_bytes.cmp(&a.memory_bytes)));
        apps
    }
}

/// Small stable (non-cryptographic) hash for application ids.
fn fxhash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

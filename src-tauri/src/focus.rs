// TENREC - Focus mode
//
// Gives one application priority over everything else the user runs, using only
// reversible scheduling hints. Every change is recorded in a Focus snapshot; ending the
// session (or the next launch after a crash) undoes it.
use crate::platform::registry::{self, Hive, RegValue};
use crate::platform::{power, qos};
use crate::protection::firewall;
use crate::platform::tuning::{self, BackgroundTuning};
use crate::processes::{ApplicationItem, ProcessItem};
use crate::protection::guard;
use crate::protection::process_ctrl::creation_time;
use crate::protection::types::{AppliedChange, ChangeRecord, ChangeStatus, SnapshotKind};
use crate::protection::{ProtectionManager, RestoreResult};
use crate::telemetry::Sample;
use crate::util::LockExt;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::sync::Mutex;

const GPU_PREF_KEY: &str = r"Software\Microsoft\DirectX\UserGpuPreferences";
const BASELINE_WINDOW: usize = 15;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FocusOptions {
    /// Put background processes in Windows efficiency mode (EcoQoS).
    pub efficiency_mode: bool,
    /// Lower Normal/Above-normal background processes to Below normal.
    pub lower_priority: bool,
    pub low_io: bool,
    pub low_memory: bool,
    /// On hybrid CPUs, steer background processes to efficiency cores.
    pub efficiency_cores: bool,
    /// Ask Windows to run the focus app on the high-performance GPU (applies on next launch of the app).
    pub prefer_gpu: bool,
    /// Application ids left untouched (voice chat, music, ...).
    pub exclusions: Vec<String>,
    /// Switch the Windows power mode while focused ("best_performance", ...).
    pub power_mode: Option<String>,
    /// Applications whose upload is capped during focus (needs admin).
    pub limit_upload_apps: Vec<String>,
    pub upload_limit_kbps: Option<u32>,
    /// Applications blocked from the network during focus (needs admin).
    pub block_apps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusPreview {
    pub app_id: String,
    pub app_name: String,
    pub executable_path: String,
    pub focus_process_count: usize,
    /// Processes that will receive background treatment now.
    pub background_process_count: usize,
    /// Processes left alone and why (grouped).
    pub skipped_protected: usize,
    pub skipped_restricted: usize,
    pub skipped_excluded: usize,
    pub hybrid_cpu: bool,
    pub efficiency_core_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusStats {
    pub samples: u64,
    /// Average summed CPU (% of total) of non-focus applications in the 30 s before start.
    pub background_cpu_before: Option<f64>,
    /// Average summed CPU of non-focus applications during the session.
    pub background_cpu_during: Option<f64>,
    pub focus_cpu_during: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusStatus {
    pub active: bool,
    pub snapshot_id: Option<String>,
    pub app_id: Option<String>,
    pub app_name: Option<String>,
    pub started_at: Option<String>,
    pub options: Option<FocusOptions>,
    pub tuned_processes: usize,
    pub failed_processes: usize,
    pub gpu_preference_set: bool,
    pub stats: FocusStats,
    pub hybrid_cpu: bool,
}

struct Session {
    snapshot_id: String,
    app_id: String,
    app_name: String,
    options: FocusOptions,
    started_at: String,
    /// (pid, creation time) already handled, successfully or not.
    seen: HashSet<(u32, u64)>,
    tuned: usize,
    failed: usize,
    gpu_set: bool,
    stats: FocusStats,
    bg_sum: f64,
    fg_sum: f64,
}

#[derive(Default)]
struct Inner {
    session: Option<Session>,
    /// Recent per-sample (app id, cpu) pairs to compute the "before" baseline.
    recent: VecDeque<Vec<(String, f64)>>,
}

pub struct FocusManager {
    inner: Mutex<Inner>,
    ecores: Option<Vec<u32>>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

enum Eligibility {
    Focus,
    Background,
    Protected,
    Restricted,
    Excluded,
}

fn classify(p: &ProcessItem, app_id: &str, exclusions: &[String]) -> Eligibility {
    if p.application_id == app_id {
        return if p.restricted { Eligibility::Restricted } else { Eligibility::Focus };
    }
    if guard::check_process(p.pid, &p.name).is_err() {
        return Eligibility::Protected;
    }
    if p.restricted || p.executable_path.is_empty() {
        return Eligibility::Restricted;
    }
    if exclusions.iter().any(|e| e == &p.application_id) {
        return Eligibility::Excluded;
    }
    Eligibility::Background
}

impl FocusManager {
    pub fn new() -> Self {
        Self { inner: Mutex::new(Inner::default()), ecores: tuning::efficiency_core_set_ids() }
    }

    pub fn hybrid_cpu(&self) -> bool {
        self.ecores.is_some()
    }

    pub fn preview(&self, sample: &Sample, app: &ApplicationItem, options: &FocusOptions) -> FocusPreview {
        let mut pv = FocusPreview {
            app_id: app.id.clone(),
            app_name: app.name.clone(),
            executable_path: app.executable_path.clone(),
            focus_process_count: 0,
            background_process_count: 0,
            skipped_protected: 0,
            skipped_restricted: 0,
            skipped_excluded: 0,
            hybrid_cpu: self.hybrid_cpu(),
            efficiency_core_count: self.ecores.as_ref().map(|v| v.len()).unwrap_or(0),
        };
        for p in sample.processes.iter().filter(|p| p.pid > 4) {
            match classify(p, &app.id, &options.exclusions) {
                Eligibility::Focus => pv.focus_process_count += 1,
                Eligibility::Background => pv.background_process_count += 1,
                Eligibility::Protected => pv.skipped_protected += 1,
                Eligibility::Restricted => pv.skipped_restricted += 1,
                Eligibility::Excluded => pv.skipped_excluded += 1,
            }
        }
        pv
    }

    pub fn status(&self) -> FocusStatus {
        let g = self.inner.lock_safe();
        match &g.session {
            Some(s) => FocusStatus {
                active: true,
                snapshot_id: Some(s.snapshot_id.clone()),
                app_id: Some(s.app_id.clone()),
                app_name: Some(s.app_name.clone()),
                started_at: Some(s.started_at.clone()),
                options: Some(s.options.clone()),
                tuned_processes: s.tuned,
                failed_processes: s.failed,
                gpu_preference_set: s.gpu_set,
                stats: s.stats.clone(),
                hybrid_cpu: self.hybrid_cpu(),
            },
            None => FocusStatus {
                active: false,
                snapshot_id: None,
                app_id: None,
                app_name: None,
                started_at: None,
                options: None,
                tuned_processes: 0,
                failed_processes: 0,
                gpu_preference_set: false,
                stats: FocusStats::default(),
                hybrid_cpu: self.hybrid_cpu(),
            },
        }
    }

    pub fn start(&self, sample: &Sample, app: &ApplicationItem, options: FocusOptions, protection: &ProtectionManager) -> Result<FocusStatus, String> {
        {
            let g = self.inner.lock_safe();
            if g.session.is_some() {
                return Err("A focus session is already running. End it first.".to_string());
            }
        }
        if app.executable_path.is_empty() {
            return Err(format!("Windows did not report a path for {}, so it cannot be the focus app.", app.name));
        }

        let needs_admin = !options.limit_upload_apps.is_empty() || !options.block_apps.is_empty();
        if needs_admin && !crate::protection::process_ctrl::is_elevated() {
            return Err("Blocking apps or limiting upload needs administrator rights. Restart TENREC with \"Run as administrator\", or turn those options off.".to_string());
        }
        if !options.limit_upload_apps.is_empty() && options.upload_limit_kbps.is_none() {
            return Err("Choose an upload limit.".to_string());
        }
        let mut initial = Vec::new();
        if let Err(e) = Self::apply_session_wide(sample, app, &options, &mut initial) {
            // Undo whatever was already applied before reporting the failure.
            let id = protection.record(SnapshotKind::Focus, format!("Focus mode (failed): {}", app.name), initial);
            let _ = protection.restore_snapshot(&id);
            return Err(e);
        }
        let mut gpu_set = false;
        if options.prefer_gpu {
            let previous = registry::read(Hive::CurrentUser, GPU_PREF_KEY, &app.executable_path)?;
            let value = RegValue::string("GpuPreference=2;");
            if previous.as_ref() != Some(&value) {
                registry::write(Hive::CurrentUser, GPU_PREF_KEY, &app.executable_path, &value)?;
                initial.push(ChangeRecord {
                    change: AppliedChange::Registry { hive: Hive::CurrentUser, key: GPU_PREF_KEY.into(), name: app.executable_path.clone(), previous },
                    description: format!("{} set to prefer the high-performance GPU", app.name),
                    status: ChangeStatus::Active,
                    note: None,
                });
            }
            gpu_set = true;
        }

        let baseline = {
            let g = self.inner.lock_safe();
            let sums: Vec<f64> = g
                .recent
                .iter()
                .map(|s| s.iter().filter(|(id, _)| id != &app.id).map(|(_, c)| c).sum())
                .collect();
            (!sums.is_empty()).then(|| sums.iter().sum::<f64>() / sums.len() as f64)
        };

        let snapshot_id = protection.record(SnapshotKind::Focus, format!("Focus mode: {}", app.name), initial);
        let _ = protection.audit("focus_start", &app.name, "focus session started", true);
        {
            let mut g = self.inner.lock_safe();
            g.session = Some(Session {
                snapshot_id,
                app_id: app.id.clone(),
                app_name: app.name.clone(),
                options,
                started_at: Utc::now().to_rfc3339(),
                seen: HashSet::new(),
                tuned: 0,
                failed: 0,
                gpu_set,
                stats: FocusStats { background_cpu_before: baseline, ..Default::default() },
                bg_sum: 0.0,
                fg_sum: 0.0,
            });
        }
        self.on_sample(sample, protection);
        Ok(self.status())
    }

    /// Power mode, network blocks and upload limits: applied once at start.
    fn apply_session_wide(sample: &Sample, app: &ApplicationItem, options: &FocusOptions, out: &mut Vec<ChangeRecord>) -> Result<(), String> {
        let record = |change: AppliedChange, description: String| ChangeRecord { change, description, status: ChangeStatus::Active, note: None };
        if let Some(mode) = options.power_mode.as_deref() {
            let target = power::mode_from_name(mode).ok_or("Unknown power mode.")?;
            let previous = power::mode_guid().ok_or("This Windows version does not expose the power mode setting.")?;
            if previous != target {
                power::set_mode_guid(target)?;
                out.push(record(AppliedChange::PowerMode { previous }, format!("Power mode set to {}", mode.replace('_', " "))));
            }
        }
        for id in &options.block_apps {
            if id == &app.id {
                return Err("The focus app cannot also be blocked.".to_string());
            }
            let Some(a) = sample.applications.iter().find(|a| &a.id == id) else { continue };
            let rule_name = firewall::new_rule_name();
            firewall::block_program(&rule_name, &a.executable_path).map_err(|e| format!("{}: {e}", a.name))?;
            out.push(record(AppliedChange::FirewallRule { rule_name }, format!("{} blocked from the network during focus", a.name)));
        }
        if let Some(kbps) = options.upload_limit_kbps {
            for id in &options.limit_upload_apps {
                let Some(a) = sample.applications.iter().find(|a| &a.id == id) else { continue };
                let exe = std::path::Path::new(&a.executable_path).file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
                let name = qos::new_policy_name();
                qos::limit_upload(&name, &exe, kbps).map_err(|e| format!("{}: {e}", a.name))?;
                out.push(record(AppliedChange::QosPolicy { name }, format!("{} upload limited to {kbps} kbit/s", a.name)));
            }
        }
        Ok(())
    }

    /// Ends the session and undoes everything it changed.
    pub fn stop(&self, protection: &ProtectionManager) -> Result<RestoreResult, String> {
        let session = self.inner.lock_safe().session.take().ok_or("No focus session is running.")?;
        let result = protection.restore_snapshot(&session.snapshot_id)?;
        let _ = protection.audit("focus_end", &session.app_name, &format!("{} restored, {} failed", result.restored, result.failed), result.failed == 0);
        Ok(result)
    }

    /// Ends the session record without undoing (used when "Undo everything" already restored it).
    pub fn forget(&self) {
        self.inner.lock_safe().session = None;
    }

    /// Called by the sampler after every sample: applies focus/background treatment to
    /// processes that appeared since the last sample and updates statistics.
    pub fn on_sample(&self, sample: &Sample, protection: &ProtectionManager) {
        let mut g = self.inner.lock_safe();
        let per_app: Vec<(String, f64)> = sample.applications.iter().map(|a| (a.id.clone(), a.cpu_percent)).collect();
        g.recent.push_back(per_app);
        while g.recent.len() > BASELINE_WINDOW {
            g.recent.pop_front();
        }
        let Some(s) = g.session.as_mut() else { return };

        let bg_opts = BackgroundTuning {
            efficiency_mode: s.options.efficiency_mode,
            lower_priority: s.options.lower_priority,
            low_io: s.options.low_io,
            low_memory: s.options.low_memory,
        };
        let ecores = if s.options.efficiency_cores { self.ecores.as_deref() } else { None };
        let any_bg = bg_opts.efficiency_mode || bg_opts.lower_priority || bg_opts.low_io || bg_opts.low_memory || ecores.is_some();

        let mut records = Vec::new();
        for p in sample.processes.iter().filter(|p| p.pid > 4) {
            let class = classify(p, &s.app_id, &s.options.exclusions);
            let is_focus = matches!(class, Eligibility::Focus);
            if !is_focus && !(any_bg && matches!(class, Eligibility::Background)) {
                continue;
            }
            let Some(ct) = creation_time(p.pid) else { continue };
            if !s.seen.insert((p.pid, ct)) {
                continue;
            }
            let result = if is_focus { tuning::apply_focus(p.pid) } else { tuning::apply_background(p.pid, bg_opts, ecores) };
            match result {
                Ok(previous) => {
                    s.tuned += 1;
                    records.push(ChangeRecord {
                        change: AppliedChange::ProcessTuning { pid: p.pid, process_name: p.name.clone(), creation_time: ct, previous },
                        description: if is_focus {
                            format!("{} (PID {}) raised to High priority", p.name, p.pid)
                        } else {
                            format!("{} (PID {}) moved to background", p.name, p.pid)
                        },
                        status: ChangeStatus::Active,
                        note: None,
                    });
                }
                Err(_) => s.failed += 1,
            }
        }

        let focus_cpu: f64 = sample.applications.iter().filter(|a| a.id == s.app_id).map(|a| a.cpu_percent).sum();
        let bg_cpu: f64 = sample.applications.iter().filter(|a| a.id != s.app_id).map(|a| a.cpu_percent).sum();
        s.stats.samples += 1;
        s.bg_sum += bg_cpu;
        s.fg_sum += focus_cpu;
        s.stats.background_cpu_during = Some(s.bg_sum / s.stats.samples as f64);
        s.stats.focus_cpu_during = Some(s.fg_sum / s.stats.samples as f64);
        let snapshot_id = s.snapshot_id.clone();
        drop(g);
        protection.append(&snapshot_id, records);
    }
}

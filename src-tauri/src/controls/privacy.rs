// TENREC - Privacy settings with observed evidence
//
// Each setting is a single registry value that the Windows Settings app or a documented
// Group Policy writes. TENREC shows the current value, can switch it to the private value
// or back to the Windows default, and records every change for undo.
use crate::network::ConnectionInfo;
use crate::platform::registry::{self, Hive, RegValue};
use crate::protection::types::{AppliedChange, ChangeRecord, ChangeStatus};
use crate::util::LockExt;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingSource {
    /// The value the Windows Settings app writes.
    SettingsApp,
    /// A documented Group Policy value.
    GroupPolicy,
}

struct Def {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    hive: Hive,
    key: &'static str,
    name: &'static str,
    private: u32,
    /// None: Windows default is "value absent" (policies).
    default: Option<u32>,
    source: SettingSource,
    note: &'static str,
}

const CDM: &str = r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager";

const DEFS: &[Def] = &[
    Def { id: "advertising_id", title: "Advertising ID", description: "Apps can use an ID to show you personalised ads.", hive: Hive::CurrentUser, key: r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo", name: "Enabled", private: 0, default: Some(1), source: SettingSource::SettingsApp, note: "" },
    Def { id: "tailored_experiences", title: "Tailored experiences", description: "Microsoft uses your diagnostic data to personalise tips, ads and recommendations.", hive: Hive::CurrentUser, key: r"Software\Microsoft\Windows\CurrentVersion\Privacy", name: "TailoredExperiencesWithDiagnosticDataEnabled", private: 0, default: Some(1), source: SettingSource::SettingsApp, note: "" },
    Def { id: "start_suggestions", title: "Suggestions in Start", description: "Occasionally show app suggestions in the Start menu.", hive: Hive::CurrentUser, key: CDM, name: "SubscribedContent-338388Enabled", private: 0, default: Some(1), source: SettingSource::SettingsApp, note: "" },
    Def { id: "windows_tips", title: "Tips and suggestions", description: "Get tips and suggestions while using Windows.", hive: Hive::CurrentUser, key: CDM, name: "SubscribedContent-338389Enabled", private: 0, default: Some(1), source: SettingSource::SettingsApp, note: "" },
    Def { id: "silent_app_installs", title: "Automatic suggested app installs", description: "Windows installs suggested apps without asking.", hive: Hive::CurrentUser, key: CDM, name: "SilentInstalledAppsEnabled", private: 0, default: Some(1), source: SettingSource::SettingsApp, note: "" },
    Def { id: "language_list", title: "Language list for websites", description: "Websites can read your language list to show local content.", hive: Hive::CurrentUser, key: r"Control Panel\International\User Profile", name: "HttpAcceptLanguageOptOut", private: 1, default: Some(0), source: SettingSource::SettingsApp, note: "" },
    Def { id: "app_launch_tracking", title: "App launch tracking", description: "Windows tracks which apps you open to improve Start and search.", hive: Hive::CurrentUser, key: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced", name: "Start_TrackProgs", private: 0, default: Some(1), source: SettingSource::SettingsApp, note: "" },
    Def { id: "online_speech", title: "Online speech recognition", description: "Voice input is sent to Microsoft's cloud for recognition.", hive: Hive::CurrentUser, key: r"Software\Microsoft\Speech_OneCore\Settings\OnlineSpeechPrivacy", name: "HasAccepted", private: 0, default: Some(1), source: SettingSource::SettingsApp, note: "" },
    Def { id: "feedback_requests", title: "Feedback requests", description: "Windows asks for feedback from time to time.", hive: Hive::CurrentUser, key: r"Software\Microsoft\Siuf\Rules", name: "NumberOfSIUFInPeriod", private: 0, default: None, source: SettingSource::SettingsApp, note: "" },
    Def { id: "web_search_in_start", title: "Web results in Start search", description: "Searches typed in Start are also sent to Bing.", hive: Hive::CurrentUser, key: r"Software\Policies\Microsoft\Windows\Explorer", name: "DisableSearchBoxSuggestions", private: 1, default: None, source: SettingSource::GroupPolicy, note: "Takes effect after you sign out and back in." },
    Def { id: "activity_history", title: "Activity history", description: "Windows records the apps and files you use as activity history.", hive: Hive::LocalMachine, key: r"SOFTWARE\Policies\Microsoft\Windows\System", name: "PublishUserActivities", private: 0, default: None, source: SettingSource::GroupPolicy, note: "" },
    Def { id: "diagnostic_data", title: "Optional diagnostic data", description: "Send optional diagnostic data (browsing, app usage, enhanced error reports) in addition to required data.", hive: Hive::LocalMachine, key: r"SOFTWARE\Policies\Microsoft\Windows\DataCollection", name: "AllowTelemetry", private: 1, default: None, source: SettingSource::GroupPolicy, note: "Limits Windows to required diagnostic data. Required data is still sent on Home and Pro editions." },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacySetting {
    pub id: String,
    pub title: String,
    pub description: String,
    pub note: Option<String>,
    pub location: String,
    pub source: SettingSource,
    /// The private choice is in effect.
    pub is_private: bool,
    /// Raw current value; None when absent (Windows default applies).
    pub current_value: Option<u32>,
    pub requires_admin: bool,
}

fn state(d: &Def) -> (Option<u32>, bool) {
    let v = registry::read(d.hive, d.key, d.name).ok().flatten().and_then(|v| v.as_dword());
    (v, v == Some(d.private))
}

pub fn list() -> Vec<PrivacySetting> {
    DEFS.iter()
        .map(|d| {
            let (current_value, is_private) = state(d);
            PrivacySetting {
                id: d.id.into(),
                title: d.title.into(),
                description: d.description.into(),
                note: (!d.note.is_empty()).then(|| d.note.to_string()),
                location: format!(r"{}\{}\{}", d.hive.label(), d.key, d.name),
                source: d.source,
                is_private,
                current_value,
                requires_admin: d.hive == Hive::LocalMachine,
            }
        })
        .collect()
}

/// Switches a setting to its private value, or back to the Windows default.
pub fn set_private(id: &str, private: bool) -> Result<ChangeRecord, String> {
    let d = DEFS.iter().find(|d| d.id == id).ok_or("Unknown setting.")?;
    let previous = registry::read(d.hive, d.key, d.name)?;
    if (previous.as_ref().and_then(|v| v.as_dword()) == Some(d.private)) == private {
        return Err("It is already set that way.".to_string());
    }
    match (private, d.default) {
        (true, _) => registry::write(d.hive, d.key, d.name, &RegValue::dword(d.private))?,
        (false, Some(def)) => registry::write(d.hive, d.key, d.name, &RegValue::dword(def))?,
        (false, None) => registry::delete(d.hive, d.key, d.name)?,
    }
    Ok(ChangeRecord {
        change: AppliedChange::Registry { hive: d.hive, key: d.key.into(), name: d.name.into(), previous },
        description: format!("{}: {}", d.title, if private { "turned off" } else { "Windows default" }),
        status: ChangeStatus::Active,
        note: None,
    })
}

/// Host-name fragments of Microsoft diagnostic and telemetry endpoints.
const TELEMETRY_HOSTS: &[&str] = &[
    "events.data.microsoft.com",
    "telemetry.microsoft.com",
    "vortex",
    "watson.microsoft.com",
    "settings-win.data.microsoft.com",
    "browser.events.data.msn.com",
];

pub fn is_telemetry_host(host: &str) -> bool {
    let h = host.to_ascii_lowercase();
    TELEMETRY_HOSTS.iter().any(|t| h.contains(t))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryObservation {
    pub hostname: String,
    pub processes: Vec<String>,
    /// Samples (every 2 s) in which a connection to this host was open.
    pub samples_seen: u64,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyEvidence {
    pub since: String,
    pub hostnames_known: usize,
    pub observations: Vec<TelemetryObservation>,
}

/// Connections to Microsoft diagnostic endpoints observed since TENREC started.
pub struct EvidenceTracker {
    since: String,
    seen: Mutex<HashMap<String, TelemetryObservation>>,
    names_known: Mutex<usize>,
}

impl Default for EvidenceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl EvidenceTracker {
    pub fn new() -> Self {
        Self { since: Utc::now().to_rfc3339(), seen: Mutex::new(HashMap::new()), names_known: Mutex::new(0) }
    }

    pub fn observe(&self, connections: &[ConnectionInfo], hostnames_known: usize) {
        *self.names_known.lock_safe() = hostnames_known;
        let now = Utc::now().to_rfc3339();
        let mut seen = self.seen.lock_safe();
        let mut this_sample: HashMap<&str, Vec<&str>> = HashMap::new();
        for c in connections {
            if let Some(h) = c.hostname.as_deref().filter(|h| is_telemetry_host(h)) {
                this_sample.entry(h).or_default().push(c.process_name.as_str());
            }
        }
        for (host, procs) in this_sample {
            let o = seen.entry(host.to_string()).or_insert_with(|| TelemetryObservation {
                hostname: host.to_string(),
                processes: Vec::new(),
                samples_seen: 0,
                first_seen: now.clone(),
                last_seen: now.clone(),
            });
            o.samples_seen += 1;
            o.last_seen = now.clone();
            for p in procs {
                if !o.processes.iter().any(|x| x == p) {
                    o.processes.push(p.to_string());
                }
            }
        }
    }

    pub fn snapshot(&self) -> PrivacyEvidence {
        let mut observations: Vec<_> = self.seen.lock_safe().values().cloned().collect();
        observations.sort_by(|a, b| b.last_seen.cmp(&a.last_seen));
        PrivacyEvidence { since: self.since.clone(), hostnames_known: *self.names_known.lock_safe(), observations }
    }
}

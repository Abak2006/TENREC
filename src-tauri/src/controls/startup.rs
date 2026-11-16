// TENREC - Startup apps (Run keys and Startup folders)
//
// Enable/disable uses the StartupApproved registry values, the same mechanism as
// Task Manager's Startup tab: the original Run entry or shortcut is never touched.
use crate::platform::registry::{self, Hive, RegValue};
use crate::processes::version_info::read_version_info;
use crate::protection::types::{AppliedChange, ChangeRecord, ChangeStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN32: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run";
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupSource {
    UserRun,
    MachineRun,
    MachineRun32,
    UserStartupFolder,
    MachineStartupFolder,
}

impl StartupSource {
    const ALL: [StartupSource; 5] = [Self::UserRun, Self::MachineRun, Self::MachineRun32, Self::UserStartupFolder, Self::MachineStartupFolder];

    fn approved(self) -> (Hive, String) {
        match self {
            Self::UserRun => (Hive::CurrentUser, format!(r"{APPROVED}\Run")),
            Self::MachineRun => (Hive::LocalMachine, format!(r"{APPROVED}\Run")),
            Self::MachineRun32 => (Hive::LocalMachine, format!(r"{APPROVED}\Run32")),
            Self::UserStartupFolder => (Hive::CurrentUser, format!(r"{APPROVED}\StartupFolder")),
            Self::MachineStartupFolder => (Hive::LocalMachine, format!(r"{APPROVED}\StartupFolder")),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::UserRun => "Registry (this user)",
            Self::MachineRun => "Registry (all users)",
            Self::MachineRun32 => "Registry (all users, 32-bit)",
            Self::UserStartupFolder => "Startup folder (this user)",
            Self::MachineStartupFolder => "Startup folder (all users)",
        }
    }

    fn all_users(self) -> bool {
        !matches!(self, Self::UserRun | Self::UserStartupFolder)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    pub id: String,
    pub name: String,
    pub command: String,
    pub executable_path: Option<String>,
    pub company: Option<String>,
    pub source: StartupSource,
    pub source_label: String,
    pub enabled: bool,
    /// Changing an all-users entry writes HKLM and needs administrator rights.
    pub requires_admin: bool,
    pub running_now: bool,
}

/// Extracts the executable from a Run command line: a quoted path, or the text up to ".exe".
pub fn executable_from_command(cmd: &str) -> Option<String> {
    let cmd = cmd.trim();
    let path = if let Some(rest) = cmd.strip_prefix('"') {
        rest.split('"').next()?.to_string()
    } else {
        let lower = cmd.to_ascii_lowercase();
        match lower.find(".exe") {
            Some(i) => cmd[..i + 4].to_string(),
            None => cmd.split_whitespace().next()?.to_string(),
        }
    };
    let expanded = expand_env(&path);
    (!expanded.is_empty()).then_some(expanded)
}

fn expand_env(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end) => {
                let var = &after[..end];
                out.push_str(&std::env::var(var).unwrap_or_else(|_| format!("%{var}%")));
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

fn startup_folder(source: StartupSource) -> Option<String> {
    match source {
        StartupSource::UserStartupFolder => std::env::var("APPDATA").ok().map(|d| format!(r"{d}\Microsoft\Windows\Start Menu\Programs\Startup")),
        StartupSource::MachineStartupFolder => std::env::var("ProgramData").ok().map(|d| format!(r"{d}\Microsoft\Windows\Start Menu\Programs\StartUp")),
        _ => None,
    }
}

/// StartupApproved data: missing or even first byte = enabled, odd = disabled.
fn is_enabled(v: &Option<RegValue>) -> bool {
    v.as_ref().and_then(|v| v.data.first()).map(|b| b % 2 == 0).unwrap_or(true)
}

fn approved_value(enabled: bool) -> RegValue {
    let mut data = vec![0u8; 12];
    if enabled {
        data[0] = 0x02;
    } else {
        data[0] = 0x03;
        // Bytes 4..12: FILETIME when it was disabled (what Task Manager writes).
        let ft = (chrono::Utc::now().timestamp() as u64 + 11_644_473_600) * 10_000_000;
        data[4..12].copy_from_slice(&ft.to_le_bytes());
    }
    RegValue::binary(&data)
}

fn item_id(source: StartupSource, name: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in format!("{source:?}:{name}").bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("st_{h:x}")
}

pub fn list(running_paths: &HashSet<String>) -> Vec<StartupItem> {
    let mut items = Vec::new();
    for source in StartupSource::ALL {
        let (ahive, akey) = source.approved();
        let approved = registry::values(ahive, &akey).unwrap_or_default();
        let state_of = |name: &str| approved.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.clone());

        let entries: Vec<(String, String)> = match source {
            StartupSource::UserRun => registry::values(Hive::CurrentUser, RUN).unwrap_or_default().into_iter().filter_map(|(n, v)| v.as_string().map(|c| (n, c))).collect(),
            StartupSource::MachineRun => registry::values(Hive::LocalMachine, RUN).unwrap_or_default().into_iter().filter_map(|(n, v)| v.as_string().map(|c| (n, c))).collect(),
            StartupSource::MachineRun32 => registry::values(Hive::LocalMachine, RUN32).unwrap_or_default().into_iter().filter_map(|(n, v)| v.as_string().map(|c| (n, c))).collect(),
            _ => startup_folder(source)
                .and_then(|dir| std::fs::read_dir(dir).ok())
                .into_iter()
                .flatten()
                .flatten()
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    (!name.eq_ignore_ascii_case("desktop.ini")).then(|| (name, e.path().to_string_lossy().to_string()))
                })
                .collect(),
        };

        for (name, command) in entries {
            let exe = if matches!(source, StartupSource::UserStartupFolder | StartupSource::MachineStartupFolder) {
                Some(command.clone())
            } else {
                executable_from_command(&command)
            };
            let company = exe.as_deref().filter(|p| p.to_ascii_lowercase().ends_with(".exe")).and_then(|p| read_version_info(p).company);
            let running_now = exe.as_deref().map(|p| running_paths.contains(&p.to_ascii_lowercase())).unwrap_or(false);
            let display = match source {
                StartupSource::UserStartupFolder | StartupSource::MachineStartupFolder => {
                    Path::new(&name).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or(name.clone())
                }
                _ => name.clone(),
            };
            items.push(StartupItem {
                id: item_id(source, &name),
                enabled: is_enabled(&state_of(&name)),
                name: display,
                command,
                executable_path: exe,
                company,
                source,
                source_label: source.label().to_string(),
                requires_admin: source.all_users(),
                running_now,
            });
        }
    }
    items.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
    items
}

/// Enables or disables an entry. Returns the change record for undo.
pub fn set_enabled(target_id: &str, enabled: bool) -> Result<ChangeRecord, String> {
    for source in StartupSource::ALL {
        let names: Vec<String> = match source {
            StartupSource::UserRun => registry::values(Hive::CurrentUser, RUN)?.into_iter().map(|(n, _)| n).collect(),
            StartupSource::MachineRun => registry::values(Hive::LocalMachine, RUN)?.into_iter().map(|(n, _)| n).collect(),
            StartupSource::MachineRun32 => registry::values(Hive::LocalMachine, RUN32)?.into_iter().map(|(n, _)| n).collect(),
            _ => startup_folder(source)
                .and_then(|d| std::fs::read_dir(d).ok())
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect(),
        };
        let Some(name) = names.into_iter().find(|n| item_id(source, n) == target_id) else { continue };
        let (hive, key) = source.approved();
        let previous = registry::read(hive, &key, &name)?;
        if is_enabled(&previous) == enabled {
            return Err("It is already in that state.".to_string());
        }
        registry::write(hive, &key, &name, &approved_value(enabled))?;
        return Ok(ChangeRecord {
            change: AppliedChange::Registry { hive, key, name: name.clone(), previous },
            description: format!("Startup item {name} {}", if enabled { "enabled" } else { "disabled" }),
            status: ChangeStatus::Active,
            note: None,
        });
    }
    Err("That startup item no longer exists.".to_string())
}

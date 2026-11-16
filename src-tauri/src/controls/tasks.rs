// TENREC - Scheduled tasks (third-party tasks only)
use crate::controls::powershell_json;
use crate::protection::types::{AppliedChange, ChangeRecord, ChangeStatus};
use serde::{Deserialize, Serialize};
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTaskItem {
    /// Full task path, e.g. "\Vendor\Updater".
    pub path: String,
    pub name: String,
    pub author: Option<String>,
    pub state: String, // "Ready" | "Running" | "Disabled" | "Queued" | "Unknown"
    pub enabled: bool,
    /// Plain-language trigger kinds ("At log on", "Daily", ...).
    pub triggers: Vec<String>,
    pub actions: Vec<String>,
    pub runs_at_logon_or_boot: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawTask {
    path: String,
    state: Option<String>,
    author: Option<String>,
    #[serde(default)]
    triggers: OneOrMany,
    #[serde(default)]
    actions: OneOrMany,
}

/// ConvertTo-Json writes a single-element array as a bare value.
#[derive(Deserialize, Default)]
#[serde(untagged)]
enum OneOrMany {
    #[default]
    None,
    One(String),
    Many(Vec<Option<String>>),
}

impl OneOrMany {
    fn into_vec(self) -> Vec<String> {
        match self {
            OneOrMany::None => Vec::new(),
            OneOrMany::One(s) => vec![s],
            OneOrMany::Many(v) => v.into_iter().flatten().collect(),
        }
    }
}

fn trigger_label(class: &str) -> &'static str {
    match class {
        "MSFT_TaskLogonTrigger" => "At log on",
        "MSFT_TaskBootTrigger" => "At startup",
        "MSFT_TaskDailyTrigger" => "Daily",
        "MSFT_TaskWeeklyTrigger" => "Weekly",
        "MSFT_TaskTimeTrigger" => "At a set time",
        "MSFT_TaskIdleTrigger" => "When idle",
        "MSFT_TaskEventTrigger" => "On an event",
        "MSFT_TaskSessionStateChangeTrigger" => "On lock/unlock or connect",
        "MSFT_TaskRegistrationTrigger" => "When created or changed",
        _ => "Other",
    }
}

/// Lists tasks outside \Microsoft\. Fixed PowerShell script, no external input.
pub fn list() -> Result<Vec<ScheduledTaskItem>, String> {
    let script = "Get-ScheduledTask | Where-Object { $_.TaskPath -notlike '\\Microsoft\\*' } | ForEach-Object { \
        [pscustomobject]@{ Path=$_.TaskPath+$_.TaskName; State=[string]$_.State; Author=$_.Author; \
        Triggers=@($_.Triggers | ForEach-Object { $_.CimClass.CimClassName }); \
        Actions=@($_.Actions | ForEach-Object { ([string]$_.Execute + ' ' + [string]$_.Arguments).Trim() }) } } | ConvertTo-Json -Depth 3 -Compress";
    let value = powershell_json(script)?;
    let raws: Vec<RawTask> = match value {
        serde_json::Value::Null => Vec::new(),
        serde_json::Value::Array(_) => serde_json::from_value(value).map_err(|e| format!("Unexpected task list format: {e}"))?,
        other => vec![serde_json::from_value(other).map_err(|e| format!("Unexpected task list format: {e}"))?],
    };
    let mut items: Vec<ScheduledTaskItem> = raws
        .into_iter()
        .map(|r| {
            let classes = r.triggers.into_vec();
            let state = r.state.unwrap_or_else(|| "Unknown".into());
            let name = r.path.rsplit('\\').next().unwrap_or(&r.path).to_string();
            ScheduledTaskItem {
                runs_at_logon_or_boot: classes.iter().any(|c| c == "MSFT_TaskLogonTrigger" || c == "MSFT_TaskBootTrigger"),
                triggers: classes.iter().map(|c| trigger_label(c).to_string()).collect(),
                actions: r.actions.into_vec(),
                enabled: state != "Disabled",
                state,
                name,
                author: r.author,
                path: r.path,
            }
        })
        .collect();
    items.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
    Ok(items)
}

fn schtasks_change(path: &str, enable: bool) -> Result<(), String> {
    let out = Command::new("schtasks.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["/Change", "/TN", path, if enable { "/ENABLE" } else { "/DISABLE" }])
        .output()
        .map_err(|e| format!("Could not start schtasks: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    if text.contains("Access is denied") || text.to_ascii_lowercase().contains("access denied") {
        Err("This task belongs to the system or another user. Run TENREC as administrator to change it.".to_string())
    } else {
        Err(format!("schtasks failed: {}", text.trim()))
    }
}

/// Enables or disables a task that appears in the current listing.
pub fn set_enabled(path: &str, enable: bool) -> Result<ChangeRecord, String> {
    // Only paths TENREC itself listed are accepted; that excludes \Microsoft\ tasks.
    let task = list()?.into_iter().find(|t| t.path == path).ok_or("That task no longer exists.")?;
    if task.enabled == enable {
        return Err("It is already in that state.".to_string());
    }
    schtasks_change(&task.path, enable)?;
    Ok(ChangeRecord {
        change: AppliedChange::ScheduledTask { path: task.path.clone(), was_enabled: task.enabled },
        description: format!("Scheduled task {} {}", task.name, if enable { "enabled" } else { "disabled" }),
        status: ChangeStatus::Active,
        note: None,
    })
}

pub fn restore(path: &str, was_enabled: bool) -> Result<(), String> {
    schtasks_change(path, was_enabled)
}

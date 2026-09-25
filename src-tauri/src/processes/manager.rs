// TENREC - Native Windows Process & Application Manager
use crate::processes::types::{ApplicationItem, ProcessItem};
use chrono::Utc;
use std::collections::HashMap;
use std::path::Path;
use sysinfo::System;

pub struct ProcessManager {
    sys: sysinfo::System,
    #[allow(dead_code)]
    publisher_cache: HashMap<String, String>,
}

impl ProcessManager {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        Self {
            sys,
            publisher_cache: HashMap::new(),
        }
    }

    // Deduce application display name and clean publisher
    fn deduce_application_identity(exe_name: &str, path_str: &str) -> (String, String) {
        let lower = exe_name.to_lowercase();
        let stem = Path::new(exe_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(exe_name);

        let (app_name, publisher) = match lower.as_str() {
            "chrome.exe" => ("Google Chrome", "Google LLC"),
            "msedge.exe" => ("Microsoft Edge", "Microsoft Corporation"),
            "brave.exe" => ("Brave Browser", "Brave Software, Inc."),
            "firefox.exe" => ("Mozilla Firefox", "Mozilla Corporation"),
            "code.exe" => ("Visual Studio Code", "Microsoft Corporation"),
            "discord.exe" => ("Discord", "Discord Inc."),
            "spotify.exe" => ("Spotify", "Spotify AB"),
            "steam.exe" | "steamservice.exe" => ("Steam", "Valve Corporation"),
            "explorer.exe" => ("Windows Explorer", "Microsoft Corporation"),
            "taskmgr.exe" => ("Task Manager", "Microsoft Corporation"),
            "cmd.exe" => ("Windows Command Processor", "Microsoft Corporation"),
            "powershell.exe" | "pwsh.exe" => ("Windows PowerShell", "Microsoft Corporation"),
            "svchost.exe" => ("Host Process for Windows Services", "Microsoft Corporation"),
            "services.exe" => ("Services and Controller app", "Microsoft Corporation"),
            "lsass.exe" => ("Local Security Authority Process", "Microsoft Corporation"),
            "csrss.exe" => ("Client Server Runtime Process", "Microsoft Corporation"),
            "smss.exe" => ("Session Manager Subsystem", "Microsoft Corporation"),
            "wininit.exe" => ("Windows Start-Up Application", "Microsoft Corporation"),
            "winlogon.exe" => ("Windows Logon Application", "Microsoft Corporation"),
            "spoolsv.exe" => ("Print Spooler Subsystem", "Microsoft Corporation"),
            "devenv.exe" => ("Visual Studio 2022", "Microsoft Corporation"),
            "git.exe" => ("Git SCM", "Software Freedom Conservancy"),
            "node.exe" => ("Node.js JavaScript Runtime", "OpenJS Foundation"),
            "cargo.exe" | "rustc.exe" => ("Rust Toolchain", "Rust Project Developers"),
            "tenrec.exe" => ("TENREC Core", "TENREC Systems"),
            _ => {
                let name = if stem.len() > 1 {
                    let mut chars = stem.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                } else {
                    stem.to_string()
                };

                let pub_name = if path_str.contains("Windows\\System32") || path_str.contains("Windows\\SysWOW64") {
                    "Microsoft Windows OS"
                } else if path_str.contains("Program Files") {
                    "Installed Software Vendor"
                } else {
                    "Unknown / Unsigned"
                };

                return (name, pub_name.to_string());
            }
        };

        (app_name.to_string(), publisher.to_string())
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    }

    pub fn get_processes(&mut self, socket_counts: &HashMap<u32, usize>) -> Vec<ProcessItem> {
        self.refresh();

        let mut items = Vec::new();
        let total_mem = self.sys.total_memory() as f64;

        for (pid, proc) in self.sys.processes() {
            let pid_u32 = pid.as_u32();
            let name = proc.name().to_string_lossy().to_string();
            let path_str = proc
                .exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| name.clone());

            let (app_name, publisher) = Self::deduce_application_identity(&name, &path_str);
            let mem_bytes = proc.memory();
            let mem_percent = if total_mem > 0.0 {
                (mem_bytes as f64 / total_mem) * 100.0
            } else {
                0.0
            };

            let conn_count = *socket_counts.get(&pid_u32).unwrap_or(&0);

            items.push(ProcessItem {
                pid: pid_u32,
                ppid: proc.parent().map(|p| p.as_u32()),
                name,
                executable_path: path_str,
                publisher,
                cpu_percent: (proc.cpu_usage() as f64).clamp(0.0, 100.0),
                memory_bytes: mem_bytes,
                memory_percent: mem_percent.clamp(0.0, 100.0),
                net_upload_bps: 0,
                net_download_bps: 0,
                connection_count: conn_count,
                start_time: proc.start_time().to_string(),
                is_elevated: false, // Default user mode unless elevated query succeeds
                application_name: app_name,
            });
        }

        items
    }

    pub fn get_applications(
        &mut self,
        processes: &[ProcessItem],
    ) -> Vec<ApplicationItem> {
        let mut app_groups: HashMap<String, Vec<&ProcessItem>> = HashMap::new();

        for p in processes {
            app_groups.entry(p.application_name.clone()).or_default().push(p);
        }

        let mut apps = Vec::new();
        let now = Utc::now().to_rfc3339();

        for (app_name, procs) in app_groups {
            let pids: Vec<u32> = procs.iter().map(|p| p.pid).collect();
            let total_cpu: f64 = procs.iter().map(|p| p.cpu_percent).sum();
            let total_mem: u64 = procs.iter().map(|p| p.memory_bytes).sum();
            let total_conns: usize = procs.iter().map(|p| p.connection_count).sum();

            let first_proc = procs.first().unwrap();
            let publisher = first_proc.publisher.clone();
            let exe_path = first_proc.executable_path.clone();

            apps.push(ApplicationItem {
                id: format!("app_{}", app_name.replace(' ', "_").to_lowercase()),
                name: app_name,
                publisher,
                executable_path: exe_path,
                process_count: procs.len(),
                pids,
                cpu_percent: total_cpu,
                memory_bytes: total_mem,
                net_upload_bps: 0,
                net_download_bps: 0,
                connection_count: total_conns,
                first_seen: now.clone(),
                last_seen: now.clone(),
                baseline_status: "normal".to_string(),
                security_status: "healthy".to_string(),
            });
        }

        apps.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());
        apps
    }
}

// TENREC - Heuristic detection and action registry
//
// Detectors only produce incidents and *proposed* actions. Nothing here changes
// system state; applying an action goes through ProtectionManager after the user
// approves it.
use crate::network::types::{ConnectionInfo, ListeningPortInfo};
use crate::processes::types::{ApplicationItem, ProcessItem};
use crate::protection::guard;
use crate::protection::process_ctrl::creation_time;
use crate::security::types::*;
use crate::util::LockExt;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;

const MAX_INCIDENTS: usize = 300;
const MAX_ACTIONS: usize = 1000;

/// Script hosts and LOLBins that rarely need to talk to the internet directly.
const SCRIPT_HOSTS: &[&str] = &[
    "powershell.exe", "pwsh.exe", "cmd.exe", "wscript.exe", "cscript.exe", "mshta.exe",
    "rundll32.exe", "regsvr32.exe", "certutil.exe", "bitsadmin.exe", "msbuild.exe", "installutil.exe",
];

/// Ports commonly used by default by remote-access tools and backdoors.
const BACKDOOR_PORTS: &[u16] = &[1337, 4444, 5554, 6666, 6667, 12345, 27374, 31337, 54321];

#[derive(Default)]
struct Inner {
    incidents: HashMap<String, SecurityIncident>,
    actions: HashMap<String, ProposedAction>,
}

#[derive(Default)]
pub struct SecurityEngine {
    inner: Mutex<Inner>,
}

struct Finding {
    key: String,
    detector: &'static str,
    title: String,
    severity: IncidentSeverity,
    app: String,
    pid: Option<u32>,
    summary: String,
    why: &'static str,
    evidence: Vec<EvidenceItem>,
    actions: Vec<ProposedAction>,
}

fn ev(label: &str, value: impl Into<String>) -> EvidenceItem {
    EvidenceItem { label: label.to_string(), value: value.into() }
}

fn block_ip_action(ip: &str, process: &str) -> ProposedAction {
    ProposedAction {
        id: String::new(),
        incident_id: None,
        title: format!("Block outbound traffic to {ip}"),
        description: format!("Adds a Windows Firewall rule that blocks all outbound connections from this PC to {ip}."),
        risk: ActionRisk::Safe,
        expected_benefit: format!("{process} and every other program lose access to {ip}."),
        potential_consequence: "Anything legitimate hosted at that address (for example a CDN shared by many sites) also becomes unreachable.".to_string(),
        requires_admin: true,
        default_selected: true,
        target: ActionTarget::BlockRemoteIp { ip: ip.to_string() },
    }
}

fn suspend_action(pid: u32, name: &str) -> Option<ProposedAction> {
    if guard::check_process(pid, name).is_err() {
        return None;
    }
    let ct = creation_time(pid)?;
    Some(ProposedAction {
        id: String::new(),
        incident_id: None,
        title: format!("Suspend {name} (PID {pid})"),
        description: "Pauses every thread of the process until you resume it from the Protection page.".to_string(),
        risk: ActionRisk::Risky,
        expected_benefit: "The process can do nothing further (no network, no file writes) while you investigate.".to_string(),
        potential_consequence: "Work in that program freezes. If it is part of something larger (an installer, a build), that may fail or time out.".to_string(),
        requires_admin: false,
        default_selected: false,
        target: ActionTarget::SuspendProcess { pid, process_name: name.to_string(), creation_time: ct },
    })
}

fn block_port_action(port: u16, protocol: &str, process: &str) -> ProposedAction {
    ProposedAction {
        id: String::new(),
        incident_id: None,
        title: format!("Block inbound {protocol} port {port}"),
        description: format!("Adds a Windows Firewall rule that blocks inbound {protocol} connections to local port {port}."),
        risk: ActionRisk::Safe,
        expected_benefit: format!("Other machines can no longer reach {process} on port {port}."),
        potential_consequence: "If the service is something you use from another device (e.g. a dev server or game host), that stops working.".to_string(),
        requires_admin: true,
        default_selected: true,
        target: ActionTarget::BlockInboundPort { port, protocol: protocol.to_string() },
    }
}

impl SecurityEngine {
    pub fn new() -> Self {
        Self::default()
    }

    fn detect(
        &self,
        processes: &[ProcessItem],
        connections: &[ConnectionInfo],
        listening: &[ListeningPortInfo],
    ) -> Vec<Finding> {
        let by_pid: HashMap<u32, &ProcessItem> = processes.iter().map(|p| (p.pid, p)).collect();
        let mut findings = Vec::new();

        for c in connections.iter().filter(|c| c.is_public_remote) {
            let proc_name = c.process_name.to_ascii_lowercase();
            let path = by_pid.get(&c.pid).map(|p| p.executable_path.as_str()).unwrap_or("");

            if SCRIPT_HOSTS.contains(&proc_name.as_str()) {
                let mut actions = vec![block_ip_action(&c.remote_address, &c.process_name)];
                actions.extend(suspend_action(c.pid, &c.process_name));
                findings.push(Finding {
                    key: format!("script_net:{}:{}", c.pid, c.remote_address),
                    detector: "script_host_public_connection",
                    title: format!("{} connected to a public address", c.process_name),
                    severity: IncidentSeverity::High,
                    app: c.application_name.clone(),
                    pid: Some(c.pid),
                    summary: format!("{} (PID {}) is connected to {}:{} ({}).", c.process_name, c.pid, c.remote_address, c.remote_port, c.state),
                    why: "Script hosts and built-in admin tools are often used by malware to download payloads or contact a command server. Package managers, update scripts and your own scripts trigger this too, so check what launched it.",
                    evidence: vec![
                        ev("Process", format!("{} (PID {})", c.process_name, c.pid)),
                        ev("Path", if path.is_empty() { "Unavailable" } else { path }),
                        ev("Remote", format!("{}:{}", c.remote_address, c.remote_port)),
                        ev("State", c.state.clone()),
                    ],
                    actions,
                });
            } else if by_pid.get(&c.pid).map(|p| p.runs_from_user_writable_location).unwrap_or(false) {
                let mut actions = vec![block_ip_action(&c.remote_address, &c.process_name)];
                actions.extend(suspend_action(c.pid, &c.process_name));
                findings.push(Finding {
                    key: format!("writable_net:{}:{}", c.pid, c.remote_address),
                    detector: "user_writable_binary_public_connection",
                    title: format!("{} (running from a download/temp folder) is online", c.process_name),
                    severity: IncidentSeverity::Medium,
                    app: c.application_name.clone(),
                    pid: Some(c.pid),
                    summary: format!("{} runs from {} and is connected to {}:{}.", c.process_name, path, c.remote_address, c.remote_port),
                    why: "Programs started from Downloads or Temp folders have not been installed. Installers and portable apps do this legitimately; malware dropped by a browser or email attachment does too.",
                    evidence: vec![
                        ev("Process", format!("{} (PID {})", c.process_name, c.pid)),
                        ev("Path", path),
                        ev("Remote", format!("{}:{}", c.remote_address, c.remote_port)),
                    ],
                    actions,
                });
            }
        }

        for l in listening.iter().filter(|l| l.all_interfaces && l.protocol == "TCP") {
            let path = by_pid.get(&l.pid).map(|p| p.executable_path.as_str()).unwrap_or("");
            let writable = by_pid.get(&l.pid).map(|p| p.runs_from_user_writable_location).unwrap_or(false);
            let backdoor = BACKDOOR_PORTS.contains(&l.port);
            if !backdoor && !writable {
                continue;
            }
            let mut actions = vec![block_port_action(l.port, &l.protocol, &l.process_name)];
            actions.extend(suspend_action(l.pid, &l.process_name));
            findings.push(Finding {
                key: format!("listen:{}:{}", l.pid, l.port),
                detector: if backdoor { "backdoor_port_listener" } else { "user_writable_binary_listener" },
                title: if backdoor {
                    format!("{} is listening on port {} (common backdoor port)", l.process_name, l.port)
                } else {
                    format!("{} (running from a download/temp folder) accepts network connections", l.process_name)
                },
                severity: if backdoor { IncidentSeverity::High } else { IncidentSeverity::Medium },
                app: l.application_name.clone(),
                pid: Some(l.pid),
                summary: format!("{} (PID {}) listens on TCP {} on all interfaces ({}).", l.process_name, l.pid, l.port, l.bind_address),
                why: if backdoor {
                    "This port is the default for several remote-access tools and backdoors. Some legitimate tools use it too (pen-testing frameworks, dev servers), so confirm you started it."
                } else {
                    "A program that was not installed is accepting connections from other machines. Portable apps and game servers do this legitimately."
                },
                evidence: vec![
                    ev("Process", format!("{} (PID {})", l.process_name, l.pid)),
                    ev("Path", if path.is_empty() { "Unavailable" } else { path }),
                    ev("Listening on", format!("{}:{}", l.bind_address, l.port)),
                ],
                actions,
            });
        }
        findings
    }

    pub fn analyze(&self, processes: &[ProcessItem], connections: &[ConnectionInfo], listening: &[ListeningPortInfo]) {
        let findings = self.detect(processes, connections, listening);
        let now = Utc::now().to_rfc3339();
        let mut g = self.inner.lock_safe();

        for inc in g.incidents.values_mut() {
            inc.currently_observed = false;
        }

        for f in findings {
            let id = format!("inc_{:x}", stable_hash(&f.key));
            if let Some(existing) = g.incidents.get_mut(&id) {
                existing.last_observed = now.clone();
                existing.observation_count += 1;
                existing.currently_observed = true;
                continue;
            }
            let mut action_ids = Vec::new();
            for (i, mut a) in f.actions.into_iter().enumerate() {
                a.id = format!("{id}_a{i}");
                a.incident_id = Some(id.clone());
                action_ids.push(a.id.clone());
                g.actions.insert(a.id.clone(), a);
            }
            g.incidents.insert(
                id.clone(),
                SecurityIncident {
                    id,
                    detector: f.detector.to_string(),
                    title: f.title,
                    severity: f.severity,
                    first_observed: now.clone(),
                    last_observed: now.clone(),
                    observation_count: 1,
                    currently_observed: true,
                    affected_application: f.app,
                    affected_pid: f.pid,
                    summary: f.summary,
                    why_detected: f.why.to_string(),
                    evidence: f.evidence,
                    proposed_action_ids: action_ids,
                    status: IncidentStatus::Active,
                },
            );
        }

        if g.incidents.len() > MAX_INCIDENTS {
            let mut old: Vec<(String, String)> = g
                .incidents
                .values()
                .filter(|i| !i.currently_observed)
                .map(|i| (i.last_observed.clone(), i.id.clone()))
                .collect();
            old.sort();
            let excess = g.incidents.len() - MAX_INCIDENTS;
            for (_, id) in old.into_iter().take(excess) {
                g.incidents.remove(&id);
                g.actions.retain(|_, a| a.incident_id.as_deref() != Some(id.as_str()));
            }
        }
    }

    pub fn get_incidents(&self) -> Vec<SecurityIncident> {
        let g = self.inner.lock_safe();
        let mut list: Vec<_> = g.incidents.values().cloned().collect();
        list.sort_by(|a, b| b.last_observed.cmp(&a.last_observed));
        list
    }

    pub fn set_incident_status(&self, id: &str, status: IncidentStatus) -> bool {
        let mut g = self.inner.lock_safe();
        match g.incidents.get_mut(id) {
            Some(i) => {
                i.status = status;
                true
            }
            None => false,
        }
    }

    pub fn get_proposed_actions(&self, incident_id: Option<&str>) -> Vec<ProposedAction> {
        let g = self.inner.lock_safe();
        let mut list: Vec<_> = g
            .actions
            .values()
            .filter(|a| incident_id.is_none() || a.incident_id.as_deref() == incident_id)
            .cloned()
            .collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    pub fn get_action(&self, id: &str) -> Option<ProposedAction> {
        self.inner.lock_safe().actions.get(id).cloned()
    }

    fn register(&self, mut a: ProposedAction) -> ProposedAction {
        let mut g = self.inner.lock_safe();
        a.id = format!("usr_{}", uuid::Uuid::new_v4().simple());
        if g.actions.len() >= MAX_ACTIONS {
            g.actions.retain(|_, x| x.incident_id.is_some());
        }
        g.actions.insert(a.id.clone(), a.clone());
        a
    }

    /// Builds a user-requested process action from the latest observed process list.
    pub fn prepare_process_action(&self, processes: &[ProcessItem], pid: u32, kind: &str) -> Result<ProposedAction, String> {
        let p = processes
            .iter()
            .find(|p| p.pid == pid)
            .ok_or_else(|| format!("Process {pid} is no longer running."))?;
        guard::check_process(p.pid, &p.name)?;
        let ct = creation_time(pid).ok_or_else(|| format!("Windows denied access to {} (PID {}). It may be protected or belong to another user; run TENREC as administrator to control it.", p.name, pid))?;
        let action = match kind {
            "priority" => ProposedAction {
                id: String::new(),
                incident_id: None,
                title: format!("Lower priority of {} (PID {})", p.name, pid),
                description: "Sets the process priority class to Idle, so it only runs when nothing else needs the CPU. The previous priority is recorded and restored on undo.".to_string(),
                risk: ActionRisk::Safe,
                expected_benefit: "Keeps the rest of the system responsive while this process keeps running.".to_string(),
                potential_consequence: "The process runs slower whenever the CPU is busy.".to_string(),
                requires_admin: false,
                default_selected: true,
                target: ActionTarget::LowerPriority { pid, process_name: p.name.clone(), creation_time: ct },
            },
            "suspend" => suspend_action(pid, &p.name).ok_or_else(|| format!("{} cannot be suspended.", p.name))?,
            other => return Err(format!("Unknown process action '{other}'.")),
        };
        Ok(self.register(action))
    }

    /// Builds a user-requested outbound block for an application's executable.
    pub fn prepare_program_block(&self, app: &ApplicationItem) -> Result<ProposedAction, String> {
        guard::check_program_path(&app.executable_path)?;
        Ok(self.register(ProposedAction {
            id: String::new(),
            incident_id: None,
            title: format!("Block internet access for {}", app.name),
            description: format!("Adds a Windows Firewall rule that blocks all outbound connections made by {}.", app.executable_path),
            risk: ActionRisk::Risky,
            expected_benefit: format!("{} can no longer send or receive data over the network.", app.name),
            potential_consequence: "Sign-in, sync, updates and any online feature of the app stop working until you undo the block.".to_string(),
            requires_admin: true,
            default_selected: false,
            target: ActionTarget::BlockProgram { path: app.executable_path.clone() },
        }))
    }
}

fn stable_hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

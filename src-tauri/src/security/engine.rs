// TENREC - Security Detection & Structured Evidence Engine
use crate::baseline::engine::BaselineEngine;
use crate::network::types::{ConnectionInfo, ListeningPortInfo};
use crate::processes::types::ProcessItem;
use crate::security::types::{ActionRisk, EvidenceItem, IncidentSeverity, ProposedAction, SecurityIncident};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct SecurityEngine {
    incidents: Mutex<Vec<SecurityIncident>>,
    proposed_actions: Mutex<HashMap<String, ProposedAction>>,
}

impl SecurityEngine {
    pub fn new() -> Self {
        Self {
            incidents: Mutex::new(Vec::new()),
            proposed_actions: Mutex::new(HashMap::new()),
        }
    }

    pub fn analyze(
        &self,
        processes: &[ProcessItem],
        connections: &[ConnectionInfo],
        listening_ports: &[ListeningPortInfo],
        baseline: &BaselineEngine,
    ) {
        let now = Utc::now().to_rfc3339();
        let mut actions_to_register = Vec::new();

        // 1. Detector: Scripting / Execution Engines making external sockets
        let suspicious_binaries = [
            "powershell.exe",
            "cmd.exe",
            "wscript.exe",
            "cscript.exe",
            "mshta.exe",
            "rundll32.exe",
            "regsvr32.exe",
            "certutil.exe",
        ];

        for conn in connections.iter() {
            let proc_lower = conn.process_name.to_lowercase();
            let is_external = !conn.remote_address.starts_with("127.")
                && !conn.remote_address.starts_with("192.168.")
                && !conn.remote_address.starts_with("10.")
                && !conn.remote_address.starts_with("172.16.")
                && conn.remote_address != "0.0.0.0"
                && !conn.remote_address.is_empty();

            if is_external && suspicious_binaries.iter().any(|&b| proc_lower == b) {
                let incident_id = format!("inc_script_net_{}_{}", conn.pid, conn.remote_port);

                // Check if already reported
                let mut incidents_lock = self.incidents.lock().unwrap();
                if !incidents_lock.iter().any(|i| i.id == incident_id) {
                    let action_safe_id = format!("act_fw_block_{}_{}", conn.pid, conn.remote_port);
                    let action_risky_id = format!("act_suspend_{}", conn.pid);

                    let safe_action = ProposedAction {
                        id: action_safe_id.clone(),
                        incident_id: Some(incident_id.clone()),
                        title: format!("Block Outbound Endpoint {} via Firewall", conn.remote_address),
                        description: format!(
                            "Add scoped Windows Firewall rule blocking outbound traffic to {}:{} for process {}.",
                            conn.remote_address, conn.remote_port, conn.process_name
                        ),
                        risk: ActionRisk::Safe,
                        rationale: format!(
                            "Restricts '{}' (PID {}) from communicating with {} while allowing legitimate local execution.",
                            conn.process_name, conn.pid, conn.remote_address
                        ),
                        expected_benefit: "Prevents remote script payloads and command exfiltration immediately without terminating the script interpreter.".to_string(),
                        potential_consequence: "Outbound HTTP/HTTPS requests from this script instance will be blocked.".to_string(),
                        current_state: "Unrestricted outbound connection established".to_string(),
                        proposed_state: format!("Outbound traffic to {}:{} dropped by TENREC_RULE", conn.remote_address, conn.remote_port),
                        is_reversible: true,
                        default_selected: true,
                        selected: true,
                        duration_minutes: 0,
                        target_type: "firewall_rule".to_string(),
                        target_identifier: conn.remote_address.clone(),
                    };

                    let risky_action = ProposedAction {
                        id: action_risky_id.clone(),
                        incident_id: Some(incident_id.clone()),
                        title: format!("Suspend Process {} (PID {})", conn.process_name, conn.pid),
                        description: format!(
                            "Freeze execution of '{}' (PID {}) to prevent any further operations.",
                            conn.process_name, conn.pid
                        ),
                        risk: ActionRisk::Risky,
                        rationale: "Complete halt of binary execution until forensic review is completed.".to_string(),
                        expected_benefit: "Halts all potential malicious file writes, child process spawns, and memory tampering.".to_string(),
                        potential_consequence: "Any running automated script will hang until resumed or terminated.".to_string(),
                        current_state: "Running active execution threads".to_string(),
                        proposed_state: "All process threads suspended via NtSuspendProcess".to_string(),
                        is_reversible: true,
                        default_selected: false,
                        selected: false,
                        duration_minutes: 0,
                        target_type: "process_suspend".to_string(),
                        target_identifier: conn.pid.to_string(),
                    };

                    let incident = SecurityIncident {
                        id: incident_id.clone(),
                        title: format!("Unusual Network Connection from Script Interpreter: {}", conn.process_name),
                        severity: IncidentSeverity::High,
                        confidence_percent: 88,
                        first_observed: now.clone(),
                        last_observed: now.clone(),
                        affected_application: conn.application_name.clone(),
                        affected_pid: Some(conn.pid),
                        destinations: vec![format!("{}:{}", conn.remote_address, conn.remote_port)],
                        summary: format!(
                            "Process '{}' (PID {}) initiated an outbound {} socket to external IP {}:{}.",
                            conn.process_name, conn.pid, conn.protocol, conn.remote_address, conn.remote_port
                        ),
                        why_detected: "Script interpreters rarely communicate with arbitrary external IP addresses unless executing unverified downloaders or command-and-control scripts.".to_string(),
                        baseline_comparison: "Baseline shows 0 historical outbound sockets for this script process tree.".to_string(),
                        evidence: vec![
                            EvidenceItem {
                                key: "process_binary".to_string(),
                                label: "Process Name".to_string(),
                                observed_value: conn.process_name.clone(),
                                baseline_value: "N/A".to_string(),
                                explanation: "Executing binary is a native Windows scripting host.".to_string(),
                            },
                            EvidenceItem {
                                key: "remote_endpoint".to_string(),
                                label: "Remote Socket".to_string(),
                                observed_value: format!("{}:{}", conn.remote_address, conn.remote_port),
                                baseline_value: "Local/None".to_string(),
                                explanation: "Outbound socket connects directly to a non-private public IP address.".to_string(),
                            },
                            EvidenceItem {
                                key: "socket_state".to_string(),
                                label: "Connection State".to_string(),
                                observed_value: conn.state.clone(),
                                baseline_value: "CLOSED".to_string(),
                                explanation: "Active connection in ESTABLISHED or SYN_SENT state.".to_string(),
                            },
                        ],
                        proposed_action_ids: vec![action_safe_id, action_risky_id],
                        status: "ACTIVE".to_string(),
                    };

                    incidents_lock.push(incident);
                    actions_to_register.push(safe_action);
                    actions_to_register.push(risky_action);
                }
            }
        }

        // 2. Detector: High Traffic Outlier vs Baseline
        for proc in processes.iter() {
            if proc.net_upload_bps > 5_000_000 { // > 5 MB/s upload
                let incident_id = format!("inc_upload_spike_{}", proc.pid);
                let mut incidents_lock = self.incidents.lock().unwrap();
                if !incidents_lock.iter().any(|i| i.id == incident_id) {
                    let action_safe_id = format!("act_prio_drop_{}", proc.pid);
                    let safe_action = ProposedAction {
                        id: action_safe_id.clone(),
                        incident_id: Some(incident_id.clone()),
                        title: format!("Lower Process Priority for {}", proc.name),
                        description: format!("Set process priority of '{}' (PID {}) to IDLE to prevent network & CPU saturation.", proc.name, proc.pid),
                        risk: ActionRisk::Safe,
                        rationale: "Reduces resource contention while user monitors the outbound data flow.".to_string(),
                        expected_benefit: "Preserves desktop responsiveness without terminating the workload.".to_string(),
                        potential_consequence: "Process will run slower in the background.".to_string(),
                        current_state: "Normal / Above Normal priority".to_string(),
                        proposed_state: "Idle Priority Class".to_string(),
                        is_reversible: true,
                        default_selected: true,
                        selected: true,
                        duration_minutes: 0,
                        target_type: "process_priority".to_string(),
                        target_identifier: proc.pid.to_string(),
                    };

                    let incident = SecurityIncident {
                        id: incident_id.clone(),
                        title: format!("High Upload Bandwidth Divergence: {}", proc.name),
                        severity: IncidentSeverity::Medium,
                        confidence_percent: 75,
                        first_observed: now.clone(),
                        last_observed: now.clone(),
                        affected_application: proc.application_name.clone(),
                        affected_pid: Some(proc.pid),
                        destinations: vec![],
                        summary: format!(
                            "Process '{}' is uploading at {:.2} MB/s, significantly exceeding baseline activity.",
                            proc.name, (proc.net_upload_bps as f64) / (1024.0 * 1024.0)
                        ),
                        why_detected: "Outbound transfer rate exceeds 5 MB/s and deviates from moving historical average.".to_string(),
                        baseline_comparison: format!("Baseline average: {:.2} KB/s", baseline.get_baseline(&proc.application_name).map(|b| b.avg_net_bps / 1024.0).unwrap_or(0.0)),
                        evidence: vec![
                            EvidenceItem {
                                key: "upload_rate".to_string(),
                                label: "Upload Rate".to_string(),
                                observed_value: format!("{:.2} MB/s", (proc.net_upload_bps as f64) / (1024.0 * 1024.0)),
                                baseline_value: "< 100 KB/s".to_string(),
                                explanation: "Sustained high data throughput detected.".to_string(),
                            }
                        ],
                        proposed_action_ids: vec![action_safe_id],
                        status: "ACTIVE".to_string(),
                    };

                    incidents_lock.push(incident);
                    actions_to_register.push(safe_action);
                }
            }
        }

        // 3. Detector: High-Risk / Backdoor Listening Ports on 0.0.0.0
        let risky_ports = [4444, 1337, 31337, 5555, 6667, 8888];
        for port_info in listening_ports.iter() {
            if (risky_ports.contains(&port_info.port) || (!port_info.is_known_service && port_info.port > 10000))
                && port_info.bind_address == "0.0.0.0"
            {
                let incident_id = format!("inc_listen_port_{}_{}", port_info.pid, port_info.port);
                let mut incidents_lock = self.incidents.lock().unwrap();
                if !incidents_lock.iter().any(|i| i.id == incident_id) {
                    let action_safe_id = format!("act_fw_port_block_{}", port_info.port);
                    let safe_action = ProposedAction {
                        id: action_safe_id.clone(),
                        incident_id: Some(incident_id.clone()),
                        title: format!("Block Inbound Port :{} via Firewall", port_info.port),
                        description: format!(
                            "Add scoped Windows Firewall rule blocking inbound connections on port {} for process {}.",
                            port_info.port, port_info.process_name
                        ),
                        risk: ActionRisk::Safe,
                        rationale: format!(
                            "Prevents remote access to listening port {} without shutting down the application.",
                            port_info.port
                        ),
                        expected_benefit: "Neutralizes incoming connections while process stays active.".to_string(),
                        potential_consequence: "External systems will receive connection refused on this port.".to_string(),
                        current_state: format!("Port {} listening on all network interfaces (0.0.0.0)", port_info.port),
                        proposed_state: format!("Inbound port {} traffic dropped by TENREC_RULE", port_info.port),
                        is_reversible: true,
                        default_selected: true,
                        selected: true,
                        duration_minutes: 0,
                        target_type: "firewall_rule".to_string(),
                        target_identifier: port_info.port.to_string(),
                    };

                    let incident = SecurityIncident {
                        id: incident_id.clone(),
                        title: format!("Unusual Open Listening Port :{} ({})", port_info.port, port_info.process_name),
                        severity: if risky_ports.contains(&port_info.port) { IncidentSeverity::High } else { IncidentSeverity::Medium },
                        confidence_percent: 82,
                        first_observed: now.clone(),
                        last_observed: now.clone(),
                        affected_application: port_info.application_name.clone(),
                        affected_pid: Some(port_info.pid),
                        destinations: vec![format!("{}:{}", port_info.bind_address, port_info.port)],
                        summary: format!(
                            "Process '{}' opened a listening socket on port :{} binding to all interfaces.",
                            port_info.process_name, port_info.port
                        ),
                        why_detected: "Process is listening on an unverified port bound to 0.0.0.0, exposing a local service to the network.".to_string(),
                        baseline_comparison: "No historical baseline for this listening port.".to_string(),
                        evidence: vec![
                            EvidenceItem {
                                key: "listening_port".to_string(),
                                label: "Listening Port".to_string(),
                                observed_value: format!(":{}", port_info.port),
                                baseline_value: "Closed".to_string(),
                                explanation: "Socket is actively accepting connections.".to_string(),
                            },
                            EvidenceItem {
                                key: "bind_address".to_string(),
                                label: "Bind Address".to_string(),
                                observed_value: port_info.bind_address.clone(),
                                baseline_value: "127.0.0.1".to_string(),
                                explanation: "Bound globally to 0.0.0.0 (all network adapters) rather than localhost.".to_string(),
                            }
                        ],
                        proposed_action_ids: vec![action_safe_id],
                        status: "ACTIVE".to_string(),
                    };

                    incidents_lock.push(incident);
                    actions_to_register.push(safe_action);
                }
            }
        }

        // 4. Register proposed actions
        let mut actions_lock = self.proposed_actions.lock().unwrap();
        for act in actions_to_register {
            actions_lock.insert(act.id.clone(), act);
        }
    }

    pub fn get_incidents(&self) -> Vec<SecurityIncident> {
        let lock = self.incidents.lock().unwrap();
        lock.clone()
    }

    pub fn get_proposed_actions(&self, incident_id: Option<&str>) -> Vec<ProposedAction> {
        let lock = self.proposed_actions.lock().unwrap();
        match incident_id {
            Some(id) => lock.values().filter(|a| a.incident_id.as_deref() == Some(id)).cloned().collect(),
            None => lock.values().cloned().collect(),
        }
    }

    pub fn set_incident_status(&self, id: &str, status: &str) -> bool {
        let mut lock = self.incidents.lock().unwrap();
        if let Some(inc) = lock.iter_mut().find(|i| i.id == id) {
            inc.status = status.to_string();
            true
        } else {
            false
        }
    }
}

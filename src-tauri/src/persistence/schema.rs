// TENREC - SQLite Persistence Schema & Migrations

pub const CREATE_SYSTEM_SAMPLES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS system_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    cpu_usage REAL NOT NULL,
    memory_used_bytes INTEGER NOT NULL,
    memory_total_bytes INTEGER NOT NULL,
    disk_read_bps REAL NOT NULL,
    disk_write_bps REAL NOT NULL,
    net_in_bps REAL NOT NULL,
    net_out_bps REAL NOT NULL,
    battery_percent REAL,
    ac_connected INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_system_samples_ts ON system_samples(timestamp);
"#;

pub const CREATE_NETWORK_CONNECTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS network_connections (
    id TEXT PRIMARY KEY,
    pid INTEGER NOT NULL,
    process_name TEXT NOT NULL,
    application_name TEXT NOT NULL,
    protocol TEXT NOT NULL,
    local_address TEXT NOT NULL,
    local_port INTEGER NOT NULL,
    remote_address TEXT NOT NULL,
    remote_port INTEGER NOT NULL,
    state TEXT NOT NULL,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    is_unusual INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_net_conn_remote ON network_connections(remote_address);
CREATE INDEX IF NOT EXISTS idx_net_conn_app ON network_connections(application_name);
"#;

pub const CREATE_SECURITY_INCIDENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS security_incidents (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    severity TEXT NOT NULL,
    confidence_percent INTEGER NOT NULL,
    first_observed TEXT NOT NULL,
    last_observed TEXT NOT NULL,
    affected_application TEXT NOT NULL,
    affected_pid INTEGER,
    destinations_json TEXT NOT NULL,
    summary TEXT NOT NULL,
    why_detected TEXT NOT NULL,
    baseline_comparison TEXT NOT NULL,
    evidence_json TEXT NOT NULL,
    proposed_action_ids_json TEXT NOT NULL,
    status TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_incidents_status ON security_incidents(status);
"#;

pub const CREATE_ACTIVE_POLICIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS active_policies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    policy_type TEXT NOT NULL,
    application_name TEXT NOT NULL,
    destination_endpoint TEXT,
    created_at TEXT NOT NULL,
    expires_at TEXT,
    reason TEXT NOT NULL,
    status TEXT NOT NULL
);
"#;

pub const CREATE_RESTORATION_SNAPSHOTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS restoration_snapshots (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    title TEXT NOT NULL,
    action_summary_json TEXT NOT NULL,
    rules_created_json TEXT NOT NULL,
    previous_power_mode TEXT,
    suspended_pids_json TEXT,
    status TEXT NOT NULL
);
"#;

pub const CREATE_AUDIT_LOG_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    actor TEXT NOT NULL,
    action_type TEXT NOT NULL,
    target TEXT NOT NULL,
    details TEXT NOT NULL,
    success INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audit_ts ON audit_log(timestamp);
"#;

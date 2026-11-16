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

pub const CREATE_PROTECTION_SNAPSHOTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS protection_snapshots (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    changes_json TEXT NOT NULL
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

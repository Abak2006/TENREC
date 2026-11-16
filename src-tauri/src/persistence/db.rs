// TENREC - SQLite Database Connection & Operations
use crate::persistence::schema::*;
use crate::protection::types::RestorationSnapshot;
use crate::util::LockExt;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: i64,
    pub timestamp: String,
    pub actor: String,
    pub action_type: String,
    pub target: String,
    pub details: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStats {
    pub sample_count: i64,
    pub snapshot_count: i64,
    pub audit_count: i64,
    pub file_size_bytes: u64,
    pub db_path: String,
}

pub struct Database {
    conn: Mutex<Connection>,
    db_path: PathBuf,
}

impl Database {
    pub fn init_default() -> Result<Self> {
        // Per-user location only: never fall back to a shared folder like ProgramData.
        let app_data = std::env::var("LOCALAPPDATA")
            .map_err(|_| rusqlite::Error::InvalidPath(PathBuf::from("%LOCALAPPDATA% is not set")))?;
        let dir = Path::new(&app_data).join("TENREC");
        fs::create_dir_all(&dir).map_err(|_| rusqlite::Error::InvalidPath(dir.clone()))?;
        let db_path = dir.join("tenrec.db");

        Self::open(&db_path)
    }

    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Performance & durability pragma optimizations for local-first desktop app
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
            "#,
        )?;

        // Execute table schemas
        conn.execute_batch(CREATE_SYSTEM_SAMPLES_TABLE)?;
        conn.execute_batch(CREATE_PROTECTION_SNAPSHOTS_TABLE)?;
        conn.execute_batch(CREATE_AUDIT_LOG_TABLE)?;

        Ok(Self {
            conn: Mutex::new(conn),
            db_path: path.to_path_buf(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_system_sample(
        &self,
        timestamp: &str,
        cpu_usage: f32,
        memory_used: u64,
        memory_total: u64,
        disk_read_bps: f64,
        disk_write_bps: f64,
        net_in_bps: f64,
        net_out_bps: f64,
        battery_percent: Option<f32>,
        ac_connected: bool,
    ) -> Result<()> {
        let conn = self.conn.lock_safe();
        conn.execute(
            r#"
            INSERT INTO system_samples (
                timestamp, cpu_usage, memory_used_bytes, memory_total_bytes,
                disk_read_bps, disk_write_bps, net_in_bps, net_out_bps,
                battery_percent, ac_connected
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                timestamp,
                cpu_usage,
                memory_used as i64,
                memory_total as i64,
                disk_read_bps,
                disk_write_bps,
                net_in_bps,
                net_out_bps,
                battery_percent,
                if ac_connected { 1 } else { 0 },
            ],
        )?;
        Ok(())
    }

    pub fn record_audit_entry(
        &self,
        actor: &str,
        action_type: &str,
        target: &str,
        details: &str,
        success: bool,
    ) -> Result<()> {
        let conn = self.conn.lock_safe();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT INTO audit_log (timestamp, actor, action_type, target, details, success)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                now,
                actor,
                action_type,
                target,
                details,
                if success { 1 } else { 0 },
            ],
        )?;
        Ok(())
    }

    pub fn get_audit_log(&self, limit: usize) -> Result<Vec<AuditEntry>> {
        let conn = self.conn.lock_safe();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, timestamp, actor, action_type, target, details, success
            FROM audit_log
            ORDER BY id DESC
            LIMIT ?1
            "#,
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(AuditEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                actor: row.get(2)?,
                action_type: row.get(3)?,
                target: row.get(4)?,
                details: row.get(5)?,
                success: row.get::<_, i32>(6)? == 1,
            })
        })?;

        let mut entries = Vec::new();
        for r in rows {
            entries.push(r?);
        }
        Ok(entries)
    }

    pub fn get_database_stats(&self) -> Result<DatabaseStats> {
        let conn = self.conn.lock_safe();
        let sample_count: i64 = conn.query_row("SELECT COUNT(*) FROM system_samples", [], |r| r.get(0))?;
        let snapshot_count: i64 = conn.query_row("SELECT COUNT(*) FROM protection_snapshots", [], |r| r.get(0))?;
        let audit_count: i64 = conn.query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))?;

        let file_size_bytes = fs::metadata(&self.db_path).map(|m| m.len()).unwrap_or(0);

        Ok(DatabaseStats {
            sample_count,
            snapshot_count,
            audit_count,
            file_size_bytes,
            db_path: self.db_path.to_string_lossy().to_string(),
        })
    }

    pub fn purge_old_telemetry(&self, retention_hours: u32) -> Result<usize> {
        let conn = self.conn.lock_safe();
        let cutoff = (chrono::Utc::now() - chrono::Duration::hours(retention_hours as i64)).to_rfc3339();

        let samples_deleted = conn.execute(
            "DELETE FROM system_samples WHERE timestamp < ?1",
            params![cutoff],
        )?;


        Ok(samples_deleted)
    }

    pub fn save_snapshot(&self, snap: &RestorationSnapshot) -> Result<()> {
        let changes = serde_json::to_string(&snap.changes).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let status = serde_json::to_value(snap.status)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        self.conn.lock_safe().execute(
            "INSERT INTO protection_snapshots (id, timestamp, title, status, changes_json) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET status = excluded.status, changes_json = excluded.changes_json",
            params![snap.id, snap.timestamp, snap.title, status, changes],
        )?;
        Ok(())
    }

    pub fn load_snapshots(&self) -> Result<Vec<RestorationSnapshot>> {
        let conn = self.conn.lock_safe();
        let mut stmt = conn.prepare("SELECT id, timestamp, title, status, changes_json FROM protection_snapshots ORDER BY timestamp")?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, String>(4)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, timestamp, title, status, changes_json) = row?;
            // Skip rows written by an incompatible version rather than failing startup.
            let (Ok(changes), Ok(status)) = (
                serde_json::from_str(&changes_json),
                serde_json::from_value(serde_json::Value::String(status)),
            ) else {
                log::warn!("Skipping unreadable snapshot {id}");
                continue;
            };
            out.push(RestorationSnapshot { id, timestamp, title, changes, status });
        }
        Ok(out)
    }
}

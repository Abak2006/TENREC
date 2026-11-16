// TENREC - Tauri shared state
use crate::persistence::Database;
use crate::protection::ProtectionManager;
use crate::telemetry::TelemetryHub;
use std::sync::Arc;

pub struct AppState {
    pub hub: Arc<TelemetryHub>,
    pub protection: ProtectionManager,
    pub db: Arc<Database>,
    /// Set when the on-disk database could not be opened and an in-memory one is used.
    pub storage_error: Option<String>,
}

impl AppState {
    pub fn new(db: Arc<Database>, storage_error: Option<String>) -> Self {
        Self {
            hub: Arc::new(TelemetryHub::new()),
            protection: ProtectionManager::new(db.clone()),
            db,
            storage_error,
        }
    }
}

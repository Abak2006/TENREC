// TENREC - Tauri shared state
use crate::focus::FocusManager;
use crate::persistence::Database;
use crate::protection::ProtectionManager;
use crate::telemetry::TelemetryHub;
use std::sync::Arc;

pub struct AppState {
    pub hub: Arc<TelemetryHub>,
    pub protection: Arc<ProtectionManager>,
    pub focus: Arc<FocusManager>,
    pub db: Arc<Database>,
    /// Set when the on-disk database could not be opened and an in-memory one is used.
    pub storage_error: Option<String>,
}

impl AppState {
    pub fn new(db: Arc<Database>, storage_error: Option<String>) -> Self {
        Self {
            hub: Arc::new(TelemetryHub::new()),
            protection: Arc::new(ProtectionManager::new(db.clone())),
            focus: Arc::new(FocusManager::new()),
            db,
            storage_error,
        }
    }
}

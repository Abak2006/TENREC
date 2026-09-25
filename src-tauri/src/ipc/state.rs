// TENREC - Tauri App Shared State
use crate::baseline::BaselineEngine;
use crate::correlation::CorrelationEngine;
use crate::network::NetworkManager;
use crate::persistence::Database;
use crate::processes::ProcessManager;
use crate::protection::ProtectionManager;
use crate::security::SecurityEngine;
use crate::system::SystemManager;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub system: Mutex<SystemManager>,
    pub processes: Mutex<ProcessManager>,
    pub network: Mutex<NetworkManager>,
    pub correlation: Arc<CorrelationEngine>,
    pub baseline: Arc<BaselineEngine>,
    pub security: Arc<SecurityEngine>,
    pub protection: Arc<ProtectionManager>,
    pub db: Arc<Database>,
}

impl AppState {
    pub fn new() -> Self {
        let db = Arc::new(Database::init_default().expect("Failed to initialize TENREC SQLite database"));
        let correlation = Arc::new(CorrelationEngine::new());
        let baseline = Arc::new(BaselineEngine::new());
        let security = Arc::new(SecurityEngine::new());
        let protection = Arc::new(ProtectionManager::new(db.clone()));

        Self {
            system: Mutex::new(SystemManager::new()),
            processes: Mutex::new(ProcessManager::new()),
            network: Mutex::new(NetworkManager::new()),
            correlation,
            baseline,
            security,
            protection,
            db,
        }
    }
}

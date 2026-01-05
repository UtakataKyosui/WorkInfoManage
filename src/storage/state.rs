use crate::config::StorageConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageType {
    Json,
    Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageState {
    pub storage_type: StorageType,
    pub storage_config: StorageConfig,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

impl StorageState {
    fn state_file_path() -> PathBuf {
        // Use dirs crate for cross-platform home directory resolution
        if let Some(home) = dirs::home_dir() {
            home.join(".task-manager-state.json")
        } else {
            PathBuf::from(".task-manager-state.json")
        }
    }

    pub fn load() -> Option<Self> {
        let path = Self::state_file_path();
        if !path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::state_file_path();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn new(storage_type: StorageType, storage_config: StorageConfig) -> Self {
        Self {
            storage_type,
            storage_config,
            last_used: chrono::Utc::now(),
        }
    }
}

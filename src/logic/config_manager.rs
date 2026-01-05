use crate::config::{Config, StorageConfig};
use anyhow::Result;
use std::path::PathBuf;

pub struct ConfigManager {
    pub config: Config,
    pub status_message: String,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        // Load raw config to preserve ${VAR} syntax for editing
        // If file doesn't exist, start with default JSON config
        let config = match Config::load_raw() {
            Ok(c) => c,
            Err(_) => {
                // Return default JSON config if file not found
                Config {
                    storage: StorageConfig::json(),
                }
            }
        };

        Ok(Self {
            config,
            status_message: String::new(),
        })
    }

    pub fn save(&mut self) {
        if let Err(e) = self.config.save() {
            self.status_message = format!("Failed to save config: {}", e);
        } else {
            self.status_message =
                "Configuration saved. Restart required to apply changes.".to_string();
        }
    }

    pub fn set_storage_type_json(&mut self) {
        // If already JSON, do nothing
        if let StorageConfig::Json { .. } = self.config.storage {
            return;
        }

        // Switch to JSON with default path (preserved)
        // We recreate the struct to reset to default path or keep existing if mapped?
        // Since we are creating from scratch, we can just use the default logic or existing logic.
        // For editing, let's just reset to default JSON setup.
        // But wait, existing Config logic has a hardcoded path.
        // Let's use the default path logic from StorageConfig::json() but we need raw path.
        // Inspecting config/mod.rs: JSON_STORAGE_PATH = "~/task-manage/data.json"

        self.config.storage = StorageConfig::Json {
            path: PathBuf::from("~/task-manage/data.json"),
        };
        self.status_message = "Switched to JSON storage. Save to persist.".to_string();
    }

    pub fn set_storage_type_database(&mut self) {
        // If already Database, do nothing
        if let StorageConfig::Database { .. } = self.config.storage {
            return;
        }

        // Switch to Database with placeholder or default
        self.config.storage = StorageConfig::Database {
            url: "${DATABASE_URL}".to_string(),
        };
        self.status_message = "Switched to Database storage. Save to persist.".to_string();
    }

    pub fn update_database_url(&mut self, url: String) {
        if let StorageConfig::Database {
            url: ref mut current_url,
        } = self.config.storage
        {
            *current_url = url;
        }
    }

    pub fn is_database(&self) -> bool {
        matches!(self.config.storage, StorageConfig::Database { .. })
    }

    pub fn get_database_url(&self) -> String {
        match &self.config.storage {
            StorageConfig::Database { url } => url.clone(),
            _ => String::new(),
        }
    }
}

use anyhow::{Context, Result};
use sea_orm::Database;
use std::sync::Arc;
use std::time::Duration;
use crate::config::{Config, StorageConfig};
use crate::storage::{Storage, json::JsonStorage, database::DatabaseStorage};

/// Create a storage backend based on configuration
pub async fn create_storage(config: &Config) -> Result<Arc<dyn Storage>> {
    match &config.storage {
        StorageConfig::Json { path } => {
            let storage = JsonStorage::new(path).await
                .context("Failed to create JSON storage")?;
            Ok(Arc::new(storage))
        }
        StorageConfig::Database { url } => {
            // Retry database connection with exponential backoff
            let max_retries = 5;
            let mut last_error = None;
            
            for attempt in 0..max_retries {
                match Database::connect(url).await {
                    Ok(db) => {
                        if attempt > 0 {
                            eprintln!("✅ Database connected successfully after {} attempt(s)", attempt + 1);
                        }
                        let storage = DatabaseStorage::new(Arc::new(db));
                        return Ok(Arc::new(storage));
                    }
                    Err(e) => {
                        last_error = Some(e);
                        if attempt < max_retries - 1 {
                            let wait_secs = 2u64.pow(attempt as u32);
                            eprintln!("⚠️  Database connection attempt {} failed. Retrying in {} seconds...", 
                                     attempt + 1, wait_secs);
                            tokio::time::sleep(Duration::from_secs(wait_secs)).await;
                        }
                    }
                }
            }
            
            Err(last_error.unwrap())
                .context(format!("Failed to connect to database after {} attempts", max_retries))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_json_storage() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("tasks.json");
        
        let config = Config {
            storage: StorageConfig::Json {
                path: path.clone(),
            },
        };
        
        let storage = create_storage(&config).await.unwrap();
        let tasks = storage.load_tasks().await.unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_create_storage_from_toml() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_tasks.json");
        
        let toml_str = format!(r#"
            [storage]
            type = "json"
            path = "{}"
        "#, path.display());
        
        let config = Config::parse(&toml_str).unwrap();
        let storage = create_storage(&config).await.unwrap();
        
        let tasks = storage.load_tasks().await.unwrap();
        assert_eq!(tasks.len(), 0);
    }
}

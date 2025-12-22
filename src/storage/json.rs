use async_trait::async_trait;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::db::{tasks, task_notes, work_logs};
use crate::storage::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonData {
    tasks: Vec<tasks::Model>,
    notes: Vec<task_notes::Model>,
    work_logs: Vec<work_logs::Model>,
}

pub struct JsonStorage {
    path: PathBuf,
    data: Arc<RwLock<JsonData>>,
}

impl JsonStorage {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .context("Failed to create storage directory")?;
        }

        // Load existing data or create new
        let data = if path.exists() {
            let content = tokio::fs::read_to_string(&path).await
                .context("Failed to read JSON file")?;
            serde_json::from_str(&content)
                .context("Failed to parse JSON file")?
        } else {
            JsonData {
                tasks: vec![],
                notes: vec![],
                work_logs: vec![],
            }
        };

        Ok(Self {
            path,
            data: Arc::new(RwLock::new(data)),
        })
    }

    /// Helper function to persist data to file without acquiring locks
    /// Caller must ensure appropriate locking
    async fn persist(&self, data: &JsonData) -> Result<()> {
        let json = serde_json::to_string_pretty(data)
            .context("Failed to serialize data")?;

        // Write atomically using a temp file
        let temp_path = self.path.with_extension("tmp");
        tokio::fs::write(&temp_path, json).await
            .context("Failed to write temp file")?;
        tokio::fs::rename(&temp_path, &self.path).await
            .context("Failed to rename temp file")?;

        Ok(())
    }
}

#[async_trait]
impl Storage for JsonStorage {
    async fn load_tasks(&self) -> Result<Vec<tasks::Model>> {
        let data = self.data.read().await;
        Ok(data.tasks.clone())
    }

    async fn save_tasks(&self, tasks: &[tasks::Model]) -> Result<()> {
        let mut data = self.data.write().await;
        data.tasks = tasks.to_vec();
        self.persist(&data).await
    }

    async fn load_notes(&self, task_id: i32) -> Result<Vec<task_notes::Model>> {
        let data = self.data.read().await;
        Ok(data.notes.iter()
            .filter(|n| n.task_id == task_id)
            .cloned()
            .collect())
    }

    async fn load_all_notes(&self) -> Result<Vec<task_notes::Model>> {
        let data = self.data.read().await;
        Ok(data.notes.clone())
    }

    async fn save_note(&self, note: &task_notes::Model) -> Result<task_notes::Model> {
        let mut data = self.data.write().await;

        // Generate new ID if needed
        let new_note = if note.id == 0 {
            let max_id = data.notes.iter().map(|n| n.id).max().unwrap_or(0);
            task_notes::Model {
                id: max_id + 1,
                ..note.clone()
            }
        } else {
            note.clone()
        };

        data.notes.push(new_note.clone());
        self.persist(&data).await?;
        Ok(new_note)
    }

    async fn load_all_work_logs(&self) -> Result<Vec<work_logs::Model>> {
        let data = self.data.read().await;
        Ok(data.work_logs.clone())
    }

    async fn save_work_log(&self, log: &work_logs::Model) -> Result<()> {
        let mut data = self.data.write().await;
        data.work_logs.push(log.clone());
        self.persist(&data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_json_storage_new_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("data").join("tasks.json");

        let storage = JsonStorage::new(&path).await.unwrap();
        let tasks = storage.load_tasks().await.unwrap();

        assert_eq!(tasks.len(), 0);
        // Directory is created, but file is only created on first save
        assert!(path.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn test_json_storage_save_and_load_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("tasks.json");

        let storage = JsonStorage::new(&path).await.unwrap();

        let test_task = tasks::Model {
            id: 1,
            asana_id: "test123".to_string(),
            title: "Test Task".to_string(),
            description: Some("Description".to_string()),
            status: "In Progress".to_string(),
            priority: Some("High".to_string()),
            due_date: None,
            github_pr_url: None,
            review_status: None,
            last_updated_at: Utc::now().naive_utc(),
        };

        storage.save_tasks(&[test_task.clone()]).await.unwrap();
        let loaded = storage.load_tasks().await.unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "Test Task");
    }

    #[tokio::test]
    async fn test_json_storage_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("tasks.json");

        // Save data
        {
            let storage = JsonStorage::new(&path).await.unwrap();
            let test_task = tasks::Model {
                id: 1,
                asana_id: "test123".to_string(),
                title: "Persistent Task".to_string(),
                description: None,
                status: "Done".to_string(),
                priority: None,
                due_date: None,
                github_pr_url: None,
                review_status: None,
                last_updated_at: Utc::now().naive_utc(),
            };
            storage.save_tasks(&[test_task]).await.unwrap();
        }

        // Load in new instance
        {
            let storage = JsonStorage::new(&path).await.unwrap();
            let loaded = storage.load_tasks().await.unwrap();

            assert_eq!(loaded.len(), 1);
            assert_eq!(loaded[0].title, "Persistent Task");
        }
    }

    #[tokio::test]
    async fn test_json_storage_notes() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("tasks.json");

        let storage = JsonStorage::new(&path).await.unwrap();

        let note = task_notes::Model {
            id: 0,
            task_id: 1,
            content: "Test note".to_string(),
            created_at: Utc::now().naive_utc(),
        };

        let saved = storage.save_note(&note).await.unwrap();
        assert_eq!(saved.id, 1);

        let loaded = storage.load_notes(1).await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].content, "Test note");
    }
}

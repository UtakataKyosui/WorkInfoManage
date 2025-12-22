use async_trait::async_trait;
use anyhow::Result;
use crate::db::{tasks, task_notes, work_logs};

pub mod json;
pub mod database;
pub mod factory;
pub mod state;
pub mod migration;

pub use factory::create_storage;
pub use state::{StorageState, StorageType};
pub use migration::migrate_storage;

/// Storage trait for task persistence
#[async_trait]
pub trait Storage: Send + Sync {
    /// Load all tasks from storage
    async fn load_tasks(&self) -> Result<Vec<tasks::Model>>;

    /// Save tasks to storage
    async fn save_tasks(&self, tasks: &[tasks::Model]) -> Result<()>;

    /// Load notes for a specific task
    async fn load_notes(&self, task_id: i32) -> Result<Vec<task_notes::Model>>;

    /// Load all notes for all tasks (to avoid N+1 queries)
    async fn load_all_notes(&self) -> Result<Vec<task_notes::Model>>;

    /// Save a note and return the saved model with ID
    async fn save_note(&self, note: &task_notes::Model) -> Result<task_notes::Model>;

    /// Load all work logs
    async fn load_all_work_logs(&self) -> Result<Vec<work_logs::Model>>;

    /// Save a work log entry
    async fn save_work_log(&self, log: &work_logs::Model) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // Mock storage for testing
    struct MockStorage {
        tasks: std::sync::Mutex<Vec<tasks::Model>>,
    }

    #[async_trait]
    impl Storage for MockStorage {
        async fn load_tasks(&self) -> Result<Vec<tasks::Model>> {
            Ok(self.tasks.lock().unwrap().clone())
        }

        async fn save_tasks(&self, tasks: &[tasks::Model]) -> Result<()> {
            *self.tasks.lock().unwrap() = tasks.to_vec();
            Ok(())
        }

        async fn load_notes(&self, _task_id: i32) -> Result<Vec<task_notes::Model>> {
            Ok(vec![])
        }

        async fn load_all_notes(&self) -> Result<Vec<task_notes::Model>> {
            Ok(vec![])
        }

        async fn save_note(&self, note: &task_notes::Model) -> Result<task_notes::Model> {
            Ok(note.clone())
        }

        async fn load_all_work_logs(&self) -> Result<Vec<work_logs::Model>> {
            Ok(vec![])
        }

        async fn save_work_log(&self, _log: &work_logs::Model) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_storage_trait_load_tasks() {
        let storage = MockStorage {
            tasks: std::sync::Mutex::new(vec![]),
        };

        let tasks = storage.load_tasks().await.unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_storage_trait_save_and_load_tasks() {
        let storage = MockStorage {
            tasks: std::sync::Mutex::new(vec![]),
        };

        let test_task = tasks::Model {
            id: 1,
            asana_id: "test123".to_string(),
            title: "Test Task".to_string(),
            description: Some("Test description".to_string()),
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
}

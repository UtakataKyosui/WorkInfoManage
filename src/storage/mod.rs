//! # ストレージシステム
//!
//! このモジュールは、タスク、ノート、作業ログの永続化を担当します。
//!
//! ## サポートされるストレージバックエンド
//!
//! ### JSONファイルストレージ (`json`)
//! - シンプルで軽量なファイルベースのストレージ
//! - データベース不要で簡単にセットアップ可能
//! - 個人利用や開発環境に最適
//!
//! ### PostgreSQLデータベース (`database`)
//! - リレーショナルデータベースによる堅牢なストレージ
//! - 複数インスタンス間でのデータ共有が可能
//! - 本番環境や大規模データに適している
//!
//! ## 設定方法
//!
//! `config.toml`でストレージタイプを指定：
//!
//! ```toml
//! # JSONストレージの例
//! [storage]
//! type = "json"
//! path = "~/task-manage/data.json"
//!
//! # データベースストレージの例
//! [storage]
//! type = "database"
//! url = "${DATABASE_URL}"
//! ```
//!
//! ## 自動マイグレーション
//!
//! ストレージタイプを変更すると、アプリケーション起動時に自動的にデータが移行されます：
//!
//! 1. **データベース → JSON**: すべてのデータがJSONファイルにエクスポート
//! 2. **JSON → データベース**: JSONファイルのデータがデータベースにインポート
//!
//! 移行は初回起動時に一度だけ実行され、元のデータは保持されます。
//!
//! ## 使用例
//!
//! ```rust,no_run
//! use work_info_manage::storage::{create_storage, StorageState};
//! use work_info_manage::config::Config;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 設定を読み込み
//!     let config = Config::load()?;
//!     
//!     // ストレージを作成（必要に応じてマイグレーション）
//!     let storage = create_storage(&config).await?;
//!     
//!     // タスクを読み込み
//!     let tasks = storage.load_tasks().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## 詳細情報
//!
//! - マイグレーションガイド: `docs/guides/migration.md`
//! - ストレージ実装の詳細: [`Storage`] トレイト

use crate::db::{task_notes, tasks, work_logs};
use anyhow::Result;
use async_trait::async_trait;

#[cfg(not(target_arch = "wasm32"))]
pub mod database;
#[cfg(not(target_arch = "wasm32"))]
pub mod factory;
#[cfg(not(target_arch = "wasm32"))]
pub mod json;
#[cfg(not(target_arch = "wasm32"))]
pub mod migration;
#[cfg(not(target_arch = "wasm32"))]
pub mod state;

#[cfg(not(target_arch = "wasm32"))]
pub use factory::create_storage;
#[cfg(not(target_arch = "wasm32"))]
pub use migration::migrate_storage;
#[cfg(not(target_arch = "wasm32"))]
pub use state::{StorageState, StorageType};

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

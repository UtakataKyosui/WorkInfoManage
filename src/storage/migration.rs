use crate::storage::{Storage, StorageType};
use anyhow::{Context, Result};
use std::sync::Arc;

/// Migrate data from one storage backend to another
pub async fn migrate_storage(
    from: Arc<dyn Storage>,
    to: Arc<dyn Storage>,
    from_type: StorageType,
    to_type: StorageType,
) -> Result<()> {
    eprintln!("🔄 Migrating data from {:?} to {:?}...", from_type, to_type);

    // Load all data from source
    let tasks = from
        .load_tasks()
        .await
        .context("Failed to load tasks from source storage")?;

    eprintln!("  ✓ Loaded {} tasks", tasks.len());

    // Load all notes using the optimized method
    let all_notes = from
        .load_all_notes()
        .await
        .context("Failed to load notes from source storage")?;

    eprintln!("  ✓ Loaded {} notes", all_notes.len());

    // Load all work logs
    let all_work_logs = from
        .load_all_work_logs()
        .await
        .context("Failed to load work logs from source storage")?;

    eprintln!("  ✓ Loaded {} work logs", all_work_logs.len());

    // Save to destination
    to.save_tasks(&tasks)
        .await
        .context("Failed to save tasks to destination storage")?;

    eprintln!("  ✓ Saved {} tasks", tasks.len());

    // Save notes
    for note in all_notes {
        to.save_note(&note)
            .await
            .context(format!("Failed to save note {}", note.id))?;
    }

    eprintln!("  ✓ Saved all notes");

    // Save work logs
    for log in all_work_logs {
        to.save_work_log(&log)
            .await
            .context(format!("Failed to save work log {}", log.id))?;
    }

    eprintln!("  ✓ Saved all work logs");
    eprintln!("✅ Migration completed successfully!");

    Ok(())
}

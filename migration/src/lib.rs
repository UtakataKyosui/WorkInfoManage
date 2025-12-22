pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20251215_000001_create_work_logs_table;
mod m20251216_000001_create_task_notes_table;
mod m20251216_000002_add_review_status_to_tasks;
mod m20251217_000001_add_created_at_to_work_logs;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20251215_000001_create_work_logs_table::Migration),
            Box::new(m20251216_000001_create_task_notes_table::Migration),
            Box::new(m20251216_000002_add_review_status_to_tasks::Migration),
            Box::new(m20251217_000001_add_created_at_to_work_logs::Migration),
        ]
    }
}

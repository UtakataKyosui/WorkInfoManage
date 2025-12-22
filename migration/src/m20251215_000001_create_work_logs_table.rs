use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WorkLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkLogs::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WorkLogs::TaskId).integer().not_null())
                    .col(
                        ColumnDef::new(WorkLogs::StartTime)
                            .timestamp()
                            .not_null(),
                    )
                    .col(ColumnDef::new(WorkLogs::EndTime).timestamp())
                    .col(ColumnDef::new(WorkLogs::DurationSeconds).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-work_logs-tasks")
                            .from(WorkLogs::Table, WorkLogs::TaskId)
                            .to(Tasks::Table, Tasks::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WorkLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum WorkLogs {
    Table,
    Id,
    TaskId,
    StartTime,
    EndTime,
    DurationSeconds,
}

#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
}

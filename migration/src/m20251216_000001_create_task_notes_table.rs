use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TaskNotes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskNotes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TaskNotes::TaskId).integer().not_null())
                    .col(ColumnDef::new(TaskNotes::Content).text().not_null())
                    .col(
                        ColumnDef::new(TaskNotes::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-task_notes-tasks")
                            .from(TaskNotes::Table, TaskNotes::TaskId)
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
            .drop_table(Table::drop().table(TaskNotes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TaskNotes {
    Table,
    Id,
    TaskId,
    Content,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
}

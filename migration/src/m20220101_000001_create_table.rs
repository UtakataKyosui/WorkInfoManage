use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(pk_auto(Tasks::Id))
                    .col(string(Tasks::AsanaId).unique_key())
                    .col(string(Tasks::Title))
                    .col(string_null(Tasks::Description))
                    .col(string(Tasks::Status))
                    .col(string_null(Tasks::Priority))
                    .col(timestamp_null(Tasks::DueDate))
                    .col(string_null(Tasks::GithubPrUrl))
                    .col(timestamp(Tasks::LastUpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tasks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
    AsanaId,
    Title,
    Description,
    Status,
    Priority,
    DueDate,
    GithubPrUrl,
    LastUpdatedAt,
}

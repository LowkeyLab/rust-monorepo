use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Games::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Games::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Games::PlayerId).string().not_null())
                    .col(ColumnDef::new(Games::Word).string().not_null())
                    .col(ColumnDef::new(Games::Guesses).json().not_null())
                    .col(ColumnDef::new(Games::Status).string().not_null())
                    .col(ColumnDef::new(Games::Score).integer().null())
                    .col(
                        ColumnDef::new(Games::MaxAttempts)
                            .integer()
                            .not_null()
                            .default(6),
                    )
                    .col(
                        ColumnDef::new(Games::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Games::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Games::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Games {
    Table,
    Id,
    PlayerId,
    Word,
    Guesses,
    Status,
    Score,
    MaxAttempts,
    CreatedAt,
    UpdatedAt,
}

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Players::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Players::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Players::Name).string().not_null())
                    .col(
                        ColumnDef::new(Players::CreatedAt)
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
            .drop_table(Table::drop().table(Players::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Players {
    Table,
    Id,
    Name,
    CreatedAt,
}

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Players::Table)
                    .drop_column(Players::Password)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Players::Table)
                    .add_column(ColumnDef::new(Players::Password).string().not_null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum Players {
    Table,
    Password,
}

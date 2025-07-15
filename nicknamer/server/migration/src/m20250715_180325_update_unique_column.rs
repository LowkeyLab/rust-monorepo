use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // First, modify the column to remove the unique constraint
        manager
            .alter_table(
                Table::alter()
                    .table(Name::Table)
                    .modify_column(ColumnDef::new(Name::DiscordId).big_integer().not_null())
                    .to_owned(),
            )
            .await?;

        // Create a new unique constraint on DiscordId + ServerId
        manager
            .create_index(
                Index::create()
                    .name("name_discord_id_server_id_unique")
                    .table(Name::Table)
                    .col(Name::DiscordId)
                    .col(Name::ServerId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the composite unique constraint
        manager
            .drop_index(
                Index::drop()
                    .name("name_discord_id_server_id_unique")
                    .table(Name::Table)
                    .to_owned(),
            )
            .await?;

        // Recreate the original unique constraint on DiscordId by adding the column with unique key
        manager
            .alter_table(
                Table::alter()
                    .table(Name::Table)
                    .modify_column(
                        ColumnDef::new(Name::DiscordId)
                            .big_integer()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Name {
    Table,
    DiscordId,
    ServerId,
}

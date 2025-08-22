use sea_orm_migration::prelude::*;

use crate::m20250822_000002_update_games_table::Games;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Rounds::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Rounds::GameId).integer().not_null())
                    .col(ColumnDef::new(Rounds::RoundNumber).integer().not_null())
                    .col(ColumnDef::new(Rounds::Guesses).json().not_null())
                    .col(
                        ColumnDef::new(Rounds::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(Index::create().col(Rounds::GameId).col(Rounds::RoundNumber))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rounds_game_id")
                            .from(Rounds::Table, Rounds::GameId)
                            .to(Games::Table, Games::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Rounds::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Rounds {
    Table,
    GameId,
    RoundNumber,
    Guesses,
    CreatedAt,
}

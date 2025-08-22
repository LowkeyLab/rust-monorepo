use sea_orm_migration::prelude::*;

use crate::m20250822_000001_create_players_table::Players;
use crate::m20250822_000002_update_games_table::Games;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GamePlayers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GamePlayers::GameId).integer().not_null())
                    .col(ColumnDef::new(GamePlayers::PlayerId).integer().not_null())
                    .col(
                        ColumnDef::new(GamePlayers::JoinedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(GamePlayers::GameId)
                            .col(GamePlayers::PlayerId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_game_players_game_id")
                            .from(GamePlayers::Table, GamePlayers::GameId)
                            .to(Games::Table, Games::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_game_players_player_id")
                            .from(GamePlayers::Table, GamePlayers::PlayerId)
                            .to(Players::Table, Players::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GamePlayers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GamePlayers {
    Table,
    GameId,
    PlayerId,
    JoinedAt,
}

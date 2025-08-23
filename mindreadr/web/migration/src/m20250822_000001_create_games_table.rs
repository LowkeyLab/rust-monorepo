use crate::sea_orm::{EnumIter, Iterable};
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::{enumeration, json, pk_auto, string, timestamp_with_time_zone};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Game {
    Table,
    Id,
    Players,
    Name,
    State,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden, EnumIter)]
pub enum GameState {
    #[iden = "waiting_for_players"]
    WaitingForPlayers,
    #[iden = "in_progress"]
    InProgress,
    #[iden = "finished"]
    Finished,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Game::Table)
                    .if_not_exists()
                    .col(pk_auto(Game::Id))
                    .col(json(Game::Players))
                    .col(string(Game::Name))
                    .col(enumeration(
                        Game::State,
                        Alias::new("state"),
                        GameState::iter(),
                    ))
                    .col(
                        timestamp_with_time_zone(Game::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Game::UpdatedAt)
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP".to_string()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Game::Table).if_exists().to_owned())
            .await
    }
}

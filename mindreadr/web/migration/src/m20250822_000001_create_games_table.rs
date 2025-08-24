use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Games {
    Table,
    Id,
    Name,
    State,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
struct GameState;

#[derive(DeriveIden, EnumIter)]
pub enum GameStateEnum {
    WaitingForPlayers,
    InProgress,
    Finished,
}

#[derive(DeriveIden)]
enum GamePlayers {
    Table,
    GameId,
    Name,
}

#[derive(DeriveIden)]
enum Rounds {
    Table,
    GameId,
    Number,
}

#[derive(DeriveIden)]
enum RoundGuesses {
    Table,
    GameId,
    RoundNumber,
    PlayerName,
    Guess,
}

const PK_ROUNDS: &str = "pk-rounds-game_id-number";
const PK_ROUND_GUESSES: &str = "pk-round_guesses-game_id-round_number-player_id";
const PK_GAME_PLAYERS: &str = "pk-game_players";
const FK_ROUNDS_TO_GAMES: &str = "fk-rounds-game_id";
const FK_ROUNDS_GUESSES_TO_GAMES: &str = "fk-round_guesses-game_id";
const FK_GAME_PLAYERS_TO_GAMES: &str = "fk-game_players-games";
const FK_ROUND_GUESSES_TO_GAME_PLAYERS: &str = "fk-round_guesses-game_players";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(GameState)
                    .values(GameStateEnum::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Games::Table)
                    .if_not_exists()
                    .col(pk_auto(Games::Id))
                    .col(string(Games::Name))
                    .col(enumeration(
                        Games::State,
                        Alias::new("game_state"),
                        GameStateEnum::iter(),
                    ))
                    .col(
                        timestamp_with_time_zone(Games::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Games::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GamePlayers::Table)
                    .if_not_exists()
                    .col(integer(GamePlayers::GameId))
                    .col(string(GamePlayers::Name))
                    .primary_key(
                        Index::create()
                            .name(PK_GAME_PLAYERS)
                            .table(GamePlayers::Table)
                            .col(GamePlayers::GameId)
                            .col(GamePlayers::Name),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_GAME_PLAYERS_TO_GAMES)
                            .from(GamePlayers::Table, GamePlayers::GameId)
                            .to(Games::Table, Games::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Rounds::Table)
                    .if_not_exists()
                    .col(integer(Rounds::GameId))
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_ROUNDS_TO_GAMES)
                            .from(Rounds::Table, Rounds::GameId)
                            .to(Games::Table, Games::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .col(integer(Rounds::Number))
                    .primary_key(
                        Index::create()
                            .name(PK_ROUNDS)
                            .table(Rounds::Table)
                            .col(Rounds::GameId)
                            .col(Rounds::Number),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RoundGuesses::Table)
                    .if_not_exists()
                    .col(integer(RoundGuesses::GameId))
                    .col(integer(RoundGuesses::RoundNumber))
                    .col(string(RoundGuesses::PlayerName))
                    .col(string(RoundGuesses::Guess))
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_ROUNDS_GUESSES_TO_GAMES)
                            .from(RoundGuesses::Table, RoundGuesses::GameId)
                            .to(Games::Table, Games::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_ROUND_GUESSES_TO_GAME_PLAYERS)
                            .from(RoundGuesses::Table, RoundGuesses::GameId)
                            .to(GamePlayers::Table, GamePlayers::GameId)
                            .from(RoundGuesses::Table, RoundGuesses::PlayerName)
                            .to(GamePlayers::Table, GamePlayers::Name)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .primary_key(
                        Index::create()
                            .name(PK_ROUND_GUESSES)
                            .table(RoundGuesses::Table)
                            .col(RoundGuesses::GameId)
                            .col(RoundGuesses::RoundNumber)
                            .col(RoundGuesses::PlayerName),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Games::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(GamePlayers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Rounds::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RoundGuesses::Table).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(PK_ROUNDS).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(PK_ROUND_GUESSES).to_owned())
            .await?;
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(FK_ROUNDS_GUESSES_TO_GAMES)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name(PK_GAME_PLAYERS)
                    .table(GamePlayers::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(ForeignKey::drop().name(FK_GAME_PLAYERS_TO_GAMES).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(Alias::new("game_state")).to_owned())
            .await?;
        manager
            .drop_foreign_key(ForeignKey::drop().name(FK_ROUNDS_TO_GAMES).to_owned())
            .await
    }
}

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
    Players,
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
    PlayerId,
    Guess,
}

const FK_ROUNDS_TO_GAMES: &str = "fk-rounds-game_id";
const ROUNDS_UNIQUE_CONSTRAINT: &str = "idx-rounds-game_id-number";
const ROUND_GUESSES_UNIQUE_CONSTRAINT: &str = "idx-round_guesses-game_id-round_number-player_id";

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
                    .col(json(Games::Players))
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
                    .table(Rounds::Table)
                    .if_not_exists()
                    .col(integer(Rounds::GameId))
                    .col(integer(Rounds::Number))
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
                    .col(integer(RoundGuesses::PlayerId))
                    .col(string(RoundGuesses::Guess))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_ROUNDS_TO_GAMES)
                    .from(Rounds::Table, Rounds::GameId)
                    .to(Games::Table, Games::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(ROUNDS_UNIQUE_CONSTRAINT)
                    .table(Rounds::Table)
                    .col(Rounds::GameId)
                    .col(Rounds::Number)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(ROUND_GUESSES_UNIQUE_CONSTRAINT)
                    .table(RoundGuesses::Table)
                    .col(RoundGuesses::GameId)
                    .col(RoundGuesses::RoundNumber)
                    .col(RoundGuesses::PlayerId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-round_guesses-game_id")
                    .from(RoundGuesses::Table, RoundGuesses::GameId)
                    .to(Games::Table, Games::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Games::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Rounds::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RoundGuesses::Table).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(ROUNDS_UNIQUE_CONSTRAINT).to_owned())
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name(ROUND_GUESSES_UNIQUE_CONSTRAINT)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(ForeignKey::drop().name(FK_ROUNDS_TO_GAMES).to_owned())
            .await
    }
}

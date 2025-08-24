//! Backend helpers for fetching games from the database and mapping them into core domain models.
use crate::server::entities;
use anyhow::Result;
use mindreadr_core::{Game, GameState, PlayerName};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("Invalid players JSON: {0}")]
    PlayersJsonError(#[from] serde_json::Error),
}

type GamesFuture<'a> = Box<dyn Future<Output = Result<Vec<Game>, Error>> + Send + 'a>;
type GameFuture<'a> = Box<dyn Future<Output = Result<Game, Error>> + Send + 'a>;

/// Returns an async function that, given a database connection, fetches all games in the
/// provided state and converts them into the core `Game` domain model.
pub fn get_games(state: GameState) -> impl Fn(&DatabaseConnection) -> Pin<GamesFuture<'_>> {
    move |db: &DatabaseConnection| {
        // Map core state to DB active enum
        let entity_state = match state {
            GameState::WaitingForPlayers => {
                entities::sea_orm_active_enums::GameState::WaitingForPlayers
            }
            GameState::InProgress => entities::sea_orm_active_enums::GameState::InProgress,
            GameState::Finished => entities::sea_orm_active_enums::GameState::Finished,
        };

        Box::pin(get_games_with_state(db, entity_state))
    }
}

/// Returns an async function that, given a database connection, creates a new game in the
/// database with the supplied `name` and returns the core `Game` domain model.
pub fn create_game(name: String) -> impl Fn(&DatabaseConnection) -> Pin<GameFuture<'_>> {
    move |db: &DatabaseConnection| Box::pin(create_game_inner(db, name.clone()))
}

async fn get_games_with_state(
    db: &DatabaseConnection,
    entity_state: entities::sea_orm_active_enums::GameState,
) -> Result<Vec<Game>, Error> {
    let games_with_state = entities::games::Entity::find()
        .filter(entities::games::Column::State.eq(entity_state))
        .all(db)
        .await?;

    let mut games = Vec::new();
    for game in games_with_state {
        let game_state = match game.state {
            entities::sea_orm_active_enums::GameState::WaitingForPlayers => {
                GameState::WaitingForPlayers
            }
            entities::sea_orm_active_enums::GameState::InProgress => GameState::InProgress,
            entities::sea_orm_active_enums::GameState::Finished => GameState::Finished,
        };

        // Expect players stored as a JSON array of strings
        let raw_players: Vec<String> = serde_json::from_value(game.players)?;
        let player_ids: Vec<PlayerName> = raw_players.into_iter().collect();

        games.push(Game {
            id: game.id as u32,
            state: game_state,
            players: player_ids,
            rounds: vec![],
            current_round: None,
        });
    }
    Ok(games)
}

async fn create_game_inner(db: &DatabaseConnection, name: String) -> Result<Game, Error> {
    let new_model = entities::games::ActiveModel {
        players: Set(serde_json::json!([])),
        name: Set(if name.trim().is_empty() {
            "New Game".to_string()
        } else {
            name
        }),
        state: Set(entities::sea_orm_active_enums::GameState::WaitingForPlayers),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(Game {
        id: new_model.id as u32,
        players: vec![],
        rounds: vec![],
        current_round: None,
        state: GameState::WaitingForPlayers,
    })
}

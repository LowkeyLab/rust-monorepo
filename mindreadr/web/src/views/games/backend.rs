use crate::server::entities;
use anyhow::Result;
use mindreadr_core::{Game, GameState, Player};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
}

type GamesFuture<'a> = Box<dyn Future<Output = Result<Vec<Game>, Error>> + Send + 'a>;

pub fn get_games(state: GameState) -> impl Fn(&DatabaseConnection) -> Pin<GamesFuture<'_>> {
    move |db: &DatabaseConnection| {
        let entity_state: entities::games::Column::State = match state {
            GameState::WaitingForPlayers => entities::games::GameState::WaitingForPlayers,
            GameState::InProgress => entities::games::GameState::InProgress,
            GameState::Finished => entities::games::GameState::Finished,
        };

        Box::pin(get_games_with_state(db, entity_state))
    }
}

async fn get_games_with_state(
    db: &DatabaseConnection,
    entity_state: entities::games::GameState,
) -> Result<Vec<Game>, Error> {
    let games_with_state = entities::games::Model::find()
        .filter(entities::games::Column::State.eq(entity_state))
        .all(db)
        .await?;
    let mut games = Vec::new();
    for (game) in games_with_state {
        let game_state = match game.state {
            entities::games::GameState::WaitingForPlayers => GameState::WaitingForPlayers,
            entities::games::GameState::InProgress => GameState::InProgress,
            entities::games::GameState::Finished => GameState::Finished,
        };
        let player_list: Vec<Player> = players
            .into_iter()
            .map(|p| Player {
                id: p.id as u32,
                name: p.name,
            })
            .collect();
        games.push(Game {
            id: game.id as u32,
            state: game_state,
            players: player_list,
            rounds: vec![],
            current_round: None,
        });
    }
    Ok(games)
}

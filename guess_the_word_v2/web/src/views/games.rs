use crate::components::{
    EmptyGamesState, ErrorMessage, GamesGrid, GamesHeader, Header, LoadingSpinner,
};
use dioxus::prelude::*;
use guess_the_word_v2_core::Player;
use guess_the_word_v2_core::{Game, GameState};
use serde::{Deserialize, Serialize};

#[component]
pub fn Games() -> Element {
    let mut games = use_signal(Vec::<GameSummary>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);

    // Load games on component mount
    use_effect(move || {
        spawn(async move {
            match get_games().await {
                Ok(live_games) => {
                    games.set(live_games);
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load games: {}", e)));
                    loading.set(false);
                }
            }
        });
    });

    let handle_create_game = move |_| {
        // TODO: Implement game creation logic
        println!("Creating new game...");
    };

    rsx! {
        Header {}
        main { class: "min-h-screen bg-gray-50 py-8",
            div { class: "max-w-6xl mx-auto px-6",
                GamesHeader {}

                if loading() {
                    LoadingSpinner { message: "Loading games...".to_string() }
                } else if let Some(error_msg) = error() {
                    ErrorMessage { message: error_msg }
                } else if games().is_empty() {
                    EmptyGamesState { on_create_game: handle_create_game }
                } else {
                    GamesGrid { games: games(), on_create_game: handle_create_game }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameSummary {
    pub id: u32,
    pub player_count: usize,
    pub state: GameState,
    pub players: Vec<Player>,
}

impl From<Game> for GameSummary {
    fn from(game: Game) -> Self {
        GameSummary {
            id: game.id,
            player_count: game.players.len(),
            state: game.state,
            players: game.players,
        }
    }
}

#[server]
async fn get_games() -> Result<Vec<GameSummary>, ServerFnError> {
    use crate::server::get_db_pool;
    let db = get_db_pool().await;
    let games = backend::get_games(GameState::WaitingForPlayers)(db).await?;
    let mut summaries = Vec::new();
    for game in games {
        summaries.push(game.into());
    }
    Ok(summaries)
}

#[cfg(feature = "server")]
mod backend {
    use crate::server::entities::games::GameState as EntityGameState;
    use crate::server::entities::{games, prelude::*};
    use anyhow::Result;
    use guess_the_word_v2_core::{Game, GameState, Player};
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
            let entity_state: EntityGameState = match state {
                GameState::WaitingForPlayers => EntityGameState::WaitingForPlayers,
                GameState::InProgress => EntityGameState::InProgress,
                GameState::Finished => EntityGameState::Finished,
            };

            Box::pin(get_games_with_state(db, entity_state))
        }
    }

    async fn get_games_with_state(
        db: &DatabaseConnection,
        entity_state: EntityGameState,
    ) -> Result<Vec<Game>, Error> {
        let games_with_state = GamesEntity::find()
            .filter(games::Column::State.eq(entity_state))
            .find_with_related(PlayersEntity)
            .all(db)
            .await?;
        let mut games = Vec::new();
        for (game, players) in games_with_state {
            let game_state = match game.state {
                EntityGameState::WaitingForPlayers => GameState::WaitingForPlayers,
                EntityGameState::InProgress => GameState::InProgress,
                EntityGameState::Finished => GameState::Finished,
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
}

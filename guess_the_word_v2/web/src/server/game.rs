use dioxus::prelude::*;
use guess_the_word_v2_core::{Game, GameState, Player};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Response structure for the games endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameSummary {
    pub id: u32,
    pub player_count: usize,
    pub state: GameState,
    pub players: Vec<Player>,
}

/// Global game manager to store all active games
pub type GameManager = Arc<Mutex<HashMap<u32, Game>>>;

/// Initialize the game manager
pub fn init_game_manager() -> GameManager {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Server function to get all currently live games
#[server(GetGames)]
pub async fn get_games() -> Result<Vec<GameSummary>, ServerFnError> {
    let game_manager = get_game_manager().await?;
    let games = game_manager.lock()?;

    let live_games: Vec<GameSummary> = games
        .values()
        .filter(|game| !matches!(game.state, GameState::Finished))
        .map(|game| GameSummary {
            id: game.id,
            player_count: game.player_count(),
            state: game.state.clone(),
            players: game.players.clone(),
        })
        .collect();

    Ok(live_games)
}

/// Server function to create a new game
#[server(CreateGame)]
pub async fn create_game() -> Result<u32, ServerFnError> {
    let game_manager = get_game_manager().await?;
    let mut games = game_manager.lock()?;

    let game_id = generate_game_id(&games);
    let game = Game::new(game_id);

    games.insert(game_id, game);

    Ok(game_id)
}

/// Server function to join a game
#[server(JoinGame)]
pub async fn join_game(game_id: u32, player_name: String) -> Result<u32, ServerFnError> {
    let game_manager = get_game_manager().await?;
    let mut games = game_manager.lock()?;

    let game = games
        .get_mut(&game_id)
        .ok_or_else(|| ServerFnError::new("Game not found"))?;

    let player_id = generate_player_id(game);
    let player = Player {
        id: player_id,
        name: player_name,
    };

    game.add_player(player)
        .map_err(|e| ServerFnError::new(format!("Failed to join game: {}", e)))?;

    Ok(player_id)
}

/// Get or initialize the global game manager
async fn get_game_manager() -> Result<GameManager, ServerFnError> {
    use_context()
}

/// Generate a unique game ID
fn generate_game_id(games: &HashMap<u32, Game>) -> u32 {
    let mut id = 1;
    while games.contains_key(&id) {
        id += 1;
    }
    id
}

/// Generate a unique player ID for a game
fn generate_player_id(game: &Game) -> u32 {
    let mut id = 1;
    while game.players.iter().any(|p| p.id == id) {
        id += 1;
    }
    id
}

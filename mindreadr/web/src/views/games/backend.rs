//! Backend helpers for fetching games from the database and mapping them into core domain models.
use crate::server::entities;
use crate::views::games::PlayerName;
use mindreadr_core::game::{Game, GameError as DomainGameError, GameState};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::future::Future;
use std::pin::Pin;

/// Error type covering database, deserialization, and domain errors when working with games.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("Game domain error: {0}")]
    Domain(#[from] DomainGameError),
}

type GamesFuture<'a> = Box<dyn Future<Output = Result<Vec<Game>, Error>> + Send + 'a>;
type GameFuture<'a> = Box<dyn Future<Output = Result<Game, Error>> + Send + 'a>;
type PlayerAddedFuture<'a> = Box<dyn Future<Output = Result<AddPlayerResult, Error>> + Send + 'a>;
/// Result of adding a player: updated game plus the newly assigned player id.
#[derive(Debug, Clone)]
pub struct AddPlayerResult {
    pub game: Game,
    pub player_id: String,
}

/// Future type returned by get_game_model curry function.
type GameModelFuture<'a> =
    Box<dyn Future<Output = Result<entities::games::Model, Error>> + Send + 'a>;

/// Returns an async function that, given a database connection, fetches all games in the
/// provided state and converts them into the core `Game` domain model.
pub fn get_games(state: GameState) -> impl Fn(&DatabaseConnection) -> Pin<GamesFuture<'_>> {
    move |db: &DatabaseConnection| {
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

/// Returns an async function that creates a new empty game and returns the core `Game` model.
pub fn create_game() -> impl Fn(&DatabaseConnection) -> Pin<GameFuture<'_>> {
    move |db: &DatabaseConnection| {
        Box::pin(async move {
            let new_model = entities::games::ActiveModel {
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
        })
    }
}

/// Returns an async function that fetches a single game by id or errors if not found.
pub fn get_game(game_id: u32) -> impl Fn(&DatabaseConnection) -> Pin<GameFuture<'_>> {
    move |db: &DatabaseConnection| Box::pin(get_game_inner(db, game_id))
}

/// Returns an async function that fetches and returns the raw database game model or errors if not found.
/// This is useful when only database persistence concerns are needed before mapping into a domain Game.
pub fn get_game_model(game_id: u32) -> impl Fn(&DatabaseConnection) -> Pin<GameModelFuture<'_>> {
    move |db: &DatabaseConnection| {
        Box::pin(async move {
            use entities::games;
            let Some(game_model) = games::Entity::find_by_id(game_id as i32).one(db).await? else {
                return Err(Error::Database(sea_orm::DbErr::RecordNotFound(format!(
                    "game {} not found",
                    game_id
                ))));
            };
            Ok(game_model)
        })
    }
}

/// Adds a player to the game, identified by game_id. Returns the updated game and the new player ID.
pub fn add_player(game_id: u32) -> impl Fn(&DatabaseConnection) -> Pin<PlayerAddedFuture<'_>> {
    move |db: &DatabaseConnection| Box::pin(add_player_inner(db, game_id))
}

/// Future type returned by get_players curry function.
type PlayersFuture<'a> = Box<dyn Future<Output = Result<Vec<PlayerName>, Error>> + Send + 'a>;

/// Returns an async function that, given a database connection, fetches the list of player
/// names for the specified game. Errors with RecordNotFound if the game does not exist.
pub fn get_players(game_id: u32) -> impl Fn(&DatabaseConnection) -> Pin<PlayersFuture<'_>> {
    move |db: &DatabaseConnection| Box::pin(get_players_inner(db, game_id))
}

async fn get_games_with_state(
    db: &DatabaseConnection,
    entity_state: entities::sea_orm_active_enums::GameState,
) -> Result<Vec<Game>, Error> {
    use entities::games;

    // Fetch games matching state.
    let game_models = games::Entity::find()
        .filter(games::Column::State.eq(entity_state))
        .all(db)
        .await?;

    let mut games_out = Vec::with_capacity(game_models.len());
    for game_model in game_models {
        let players = get_players_inner(db, game_model.id as u32).await?;
        games_out.push(Game {
            id: game_model.id as u32,
            state: game_model.state.into(),
            players,
            rounds: vec![],
            current_round: None,
        });
    }
    Ok(games_out)
}

/// Fetch a single game by id, returning a domain `Game` or an error if not found.
async fn get_game_inner(db: &DatabaseConnection, game_id: u32) -> Result<Game, Error> {
    use entities::{game_players, games};

    // Perform a single query joining the game with its players.
    let mut results = games::Entity::find()
        .filter(games::Column::Id.eq(game_id as i32))
        .find_with_related(game_players::Entity)
        .all(db)
        .await?;

    let Some((game_model, player_models)) = results.pop() else {
        return Err(Error::Database(sea_orm::DbErr::RecordNotFound(format!(
            "game {} not found",
            game_id
        ))));
    };

    // Map player models into player name list, keeping previous alphabetical ordering behavior.
    let mut players: Vec<PlayerName> = player_models.into_iter().map(|p| p.name).collect();
    players.sort();

    Ok(Game {
        id: game_model.id as u32,
        state: game_model.state.into(),
        players,
        rounds: vec![],
        current_round: None,
    })
}

impl From<entities::sea_orm_active_enums::GameState> for GameState {
    fn from(state: entities::sea_orm_active_enums::GameState) -> Self {
        match state {
            entities::sea_orm_active_enums::GameState::WaitingForPlayers => {
                GameState::WaitingForPlayers
            }
            entities::sea_orm_active_enums::GameState::InProgress => GameState::InProgress,
            entities::sea_orm_active_enums::GameState::Finished => GameState::Finished,
        }
    }
}

async fn add_player_inner(db: &DatabaseConnection, game_id: u32) -> Result<AddPlayerResult, Error> {
    use entities::{game_players, games};
    use sea_orm::ActiveValue::Set;

    let game_model = get_game_model(game_id)(db).await?;

    // Get existing players through the shared helper.
    let existing_names = get_players_inner(db, game_id).await?;

    let mut domain_game = Game::new(game_model.id as u32);
    domain_game.players = existing_names;
    let db_entity_state = game_model.state.clone();
    domain_game.state = db_entity_state.into();

    let new_player_id = domain_game.add_player()?;

    let player_active = game_players::ActiveModel {
        game_id: Set(game_model.id),
        name: Set(new_player_id.clone()),
    };
    player_active.insert(db).await?;

    let current_state: GameState = game_model.state.clone().into();
    if domain_game.state != current_state {
        let mut active: games::ActiveModel = game_model.into();
        active.state = Set(match domain_game.state {
            GameState::WaitingForPlayers => {
                entities::sea_orm_active_enums::GameState::WaitingForPlayers
            }
            GameState::InProgress => entities::sea_orm_active_enums::GameState::InProgress,
            GameState::Finished => entities::sea_orm_active_enums::GameState::Finished,
        });
        active.update(db).await?;
    }

    Ok(AddPlayerResult {
        game: domain_game,
        player_id: new_player_id,
    })
}

async fn get_players_inner(
    db: &DatabaseConnection,
    game_id: u32,
) -> Result<Vec<PlayerName>, Error> {
    use entities::game_players;
    use sea_orm::QueryOrder;

    // Ensure game exists via shared helper.
    get_game_model(game_id)(db).await?;

    let players = game_players::Entity::find()
        .filter(game_players::Column::GameId.eq(game_id as i32))
        .order_by_asc(game_players::Column::Name)
        .all(db)
        .await?
        .into_iter()
        .map(|p| p.name)
        .collect();

    Ok(players)
}

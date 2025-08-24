use std::collections::HashMap;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Public alias representing a player's name/identifier.
pub type PlayerName = String;

/// Represents a single game instance containing players, rounds, and state.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Game {
    pub id: u32,
    pub players: Vec<PlayerName>,
    pub rounds: Vec<Round>,
    pub current_round: Option<Round>,
    pub state: GameState,
}

/// All possible states a game can be in during its lifecycle.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GameState {
    WaitingForPlayers,
    InProgress,
    Finished,
}

/// A completed or in-progress round storing each player's guess.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Round {
    pub guesses: HashMap<PlayerName, String>,
}

#[derive(Error, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GameError {
    #[error("Game is full - only 2 players are allowed")]
    GameFull,
    #[error("Game is not in progress")]
    GameNotInProgress,
    #[error("Player not found in game")]
    PlayerNotFound,
    #[error("No active round")]
    NoActiveRound,
}

impl Game {
    /// Creates a new game with the given ID
    pub fn new(id: u32) -> Self {
        Game {
            id,
            players: Vec::new(),
            rounds: Vec::new(),
            current_round: None,
            state: GameState::WaitingForPlayers,
        }
    }

    /// Adds a player to the game and starts the game if we have 2 players
    /// Returns the player ID in the form "Player1" or "Player2"
    /// Returns an error if the game already has 2 players
    pub fn add_player(&mut self) -> Result<String, GameError> {
        if self.players.len() >= 2 {
            return Err(GameError::GameFull);
        }

        let player_id = format!("Player{}", self.players.len() + 1);
        self.players.push(player_id.clone());

        // Start the game when we have exactly 2 players
        if self.players.len() == 2 && matches!(self.state, GameState::WaitingForPlayers) {
            self.start_game();
        }

        Ok(player_id)
    }

    /// Starts the game by changing state to InProgress
    pub fn start_game(&mut self) {
        self.state = GameState::InProgress;
    }

    /// Starts a new round
    pub fn start_round(&mut self) {
        let round = Round {
            guesses: HashMap::new(),
        };
        self.current_round = Some(round);
    }

    /// Ends the current round and moves it to completed rounds
    pub fn end_round(&mut self) {
        if let Some(round) = self.current_round.take() {
            self.rounds.push(round);
        }
    }

    /// Ends the game by setting state to Finished
    pub fn end_game(&mut self) {
        self.state = GameState::Finished;
        // End the current round if there is one
        self.end_round();
    }

    /// Submits a guess for a player in the current round
    pub fn submit_guess(&mut self, player: PlayerName, guess: String) -> Result<(), GameError> {
        if !matches!(self.state, GameState::InProgress) {
            return Err(GameError::GameNotInProgress);
        }

        if !self.players.contains(&player) {
            return Err(GameError::PlayerNotFound);
        }

        let current_round = self
            .current_round
            .as_mut()
            .ok_or(GameError::NoActiveRound)?;

        current_round.guesses.insert(player, guess);

        // Check if both players have made guesses
        if current_round.guesses.len() == 2 {
            let guesses: Vec<&String> = current_round.guesses.values().collect();

            // If both players guessed the same word, end the game
            if guesses[0] == guesses[1] {
                self.end_game();
            }
        }

        Ok(())
    }

    /// Returns whether the game has ended
    pub fn has_ended(&self) -> bool {
        matches!(self.state, GameState::Finished)
    }

    /// Gets the current game state
    pub fn get_state(&self) -> &GameState {
        &self.state
    }

    /// Gets the number of players in the game
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Gets the guesses for the active round, if one exists.
    pub fn get_current_round_guesses(&self) -> Result<&HashMap<PlayerName, String>, GameError> {
        let current_round = self
            .current_round
            .as_ref()
            .ok_or(GameError::NoActiveRound)?;
        Ok(&current_round.guesses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_game() {
        let game = Game::new(1);

        let expected = Game {
            id: 1,
            players: Vec::new(),
            rounds: Vec::new(),
            current_round: None,
            state: GameState::WaitingForPlayers,
        };

        assert_eq!(game, expected);
    }

    #[test]
    fn can_add_single_player_without_starting_game() {
        let mut game = Game::new(1);

        let player_id = game.add_player().unwrap();

        assert_eq!(game.player_count(), 1);
        assert_eq!(game.get_state(), &GameState::WaitingForPlayers);
        assert_eq!(player_id, "Player1");
        assert_eq!(game.players[0], "Player1".to_string());
    }

    #[test]
    fn can_start_game_when_two_players_added() {
        let mut game = Game::new(1);

        let player1_id = game.add_player().unwrap();
        assert_eq!(game.get_state(), &GameState::WaitingForPlayers);
        assert_eq!(player1_id, "Player1");

        let player2_id = game.add_player().unwrap();

        assert_eq!(game.player_count(), 2);
        assert_eq!(game.get_state(), &GameState::InProgress);
        assert_eq!(player2_id, "Player2");
        assert_eq!(game.players[0], "Player1".to_string());
        assert_eq!(game.players[1], "Player2".to_string());
    }

    #[test]
    fn cannot_add_third_player_to_full_game() {
        let mut game = Game::new(1);

        game.add_player().unwrap();
        game.add_player().unwrap();
        assert_eq!(game.get_state(), &GameState::InProgress);

        // Adding a third player should return an error
        let result = game.add_player();

        assert_eq!(result, Err(GameError::GameFull));
        assert_eq!(game.player_count(), 2);
        assert_eq!(game.get_state(), &GameState::InProgress);
    }

    #[test]
    fn cannot_add_player_to_game_with_two_players_waiting() {
        let mut game = Game::new(1);

        // Manually start the game first to keep it in WaitingForPlayers state
        game.add_player().unwrap();
        game.add_player().unwrap();

        // Try to add third player - should fail
        let result = game.add_player();

        assert_eq!(result, Err(GameError::GameFull));
        assert_eq!(game.player_count(), 2);
    }

    #[test]
    fn can_manually_start_game() {
        let mut game = Game::new(1);

        game.start_game();

        assert_eq!(game.get_state(), &GameState::InProgress);
    }

    #[test]
    fn can_start_and_end_round() {
        let mut game = Game::new(1);

        game.start_round();
        assert!(game.current_round.is_some());

        game.end_round();
        assert!(game.current_round.is_none());
        assert_eq!(game.rounds.len(), 1);
    }

    #[test]
    fn can_submit_guesses_and_end_game() {
        let mut game = Game::new(1);

        game.add_player().unwrap();
        game.add_player().unwrap();
        game.start_round();

        let player1 = "Player1".to_string();
        let player2 = "Player2".to_string();

        game.submit_guess(player1.clone(), "apple".to_string())
            .unwrap();
        assert_eq!(game.get_current_round_guesses().unwrap().len(), 1);
        assert_eq!(game.get_state(), &GameState::InProgress);

        game.submit_guess(player2.clone(), "banana".to_string())
            .unwrap();
        assert_eq!(game.get_current_round_guesses().unwrap().len(), 2);
        assert_eq!(game.get_state(), &GameState::InProgress);

        game.start_round();
        game.submit_guess(player1.clone(), "orange".to_string())
            .unwrap();
        game.submit_guess(player2.clone(), "orange".to_string())
            .unwrap();
        assert_eq!(game.get_state(), &GameState::Finished);
    }

    #[test]
    fn cannot_submit_guess_if_game_not_in_progress() {
        let mut game = Game::new(1);

        game.add_player().unwrap();
        let player = "Player1".to_string();

        let result = game.submit_guess(player, "apple".to_string());

        assert_eq!(result, Err(GameError::GameNotInProgress));
    }

    #[test]
    fn cannot_submit_guess_if_player_not_in_game() {
        let mut game = Game::new(1);

        game.add_player().unwrap();
        game.add_player().unwrap();

        game.start_round();

        let player3 = "charlie".to_string();
        let result = game.submit_guess(player3, "apple".to_string());

        assert_eq!(result, Err(GameError::PlayerNotFound));
    }

    #[test]
    fn cannot_submit_guess_if_no_active_round() {
        let mut game = Game::new(1);

        game.add_player().unwrap();
        game.start_game();

        let player = "Player1".to_string();
        let result = game.submit_guess(player, "apple".to_string());

        assert_eq!(result, Err(GameError::NoActiveRound));
    }

    #[test]
    fn ends_game_when_both_players_guess_same_word() {
        let mut game = Game::new(1);

        game.add_player().unwrap();
        game.add_player().unwrap();
        game.start_round();

        let player1 = "Player1".to_string();
        let player2 = "Player2".to_string();

        game.submit_guess(player1.clone(), "orange".to_string())
            .unwrap();
        game.submit_guess(player2.clone(), "orange".to_string())
            .unwrap();

        assert!(game.has_ended());
        assert_eq!(game.get_state(), &GameState::Finished);
    }
}

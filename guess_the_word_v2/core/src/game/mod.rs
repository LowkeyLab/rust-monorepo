use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
struct Game {
    id: u32,
    players: Vec<Player>,
    rounds: Vec<Round>,
    current_round: Option<Round>,
    state: GameState,
}

#[derive(Debug, Clone, PartialEq)]
enum GameState {
    WaitingForPlayers,
    InProgress,
    Finished,
}

#[derive(Debug, Clone, PartialEq)]
struct Round {
    guesses: HashMap<Player, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Player {
    id: u32,
    name: String,
}

#[derive(Error, Debug, Clone, PartialEq)]
enum GameError {
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
    fn new(id: u32) -> Self {
        Game {
            id,
            players: Vec::new(),
            rounds: Vec::new(),
            current_round: None,
            state: GameState::WaitingForPlayers,
        }
    }

    /// Adds a player to the game and starts the game if we have 2 players
    /// Returns an error if the game already has 2 players
    fn add_player(&mut self, player: Player) -> Result<(), GameError> {
        if self.players.len() >= 2 {
            return Err(GameError::GameFull);
        }

        self.players.push(player);

        // Start the game when we have exactly 2 players
        if self.players.len() == 2 && matches!(self.state, GameState::WaitingForPlayers) {
            self.start_game();
        }

        Ok(())
    }

    /// Starts the game by changing state to InProgress
    fn start_game(&mut self) {
        self.state = GameState::InProgress;
    }

    /// Starts a new round
    fn start_round(&mut self) {
        let round = Round {
            guesses: HashMap::new(),
        };
        self.current_round = Some(round);
    }

    /// Ends the current round and moves it to completed rounds
    fn end_round(&mut self) {
        if let Some(round) = self.current_round.take() {
            self.rounds.push(round);
        }
    }

    /// Ends the game by setting state to Finished
    fn end_game(&mut self) {
        self.state = GameState::Finished;
        // End the current round if there is one
        self.end_round();
    }

    /// Submits a guess for a player in the current round
    /// Returns true if the game should end (both players guessed the same word)
    fn submit_guess(&mut self, player: Player, guess: String) -> Result<bool, GameError> {
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
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Gets the current game state
    fn get_state(&self) -> &GameState {
        &self.state
    }

    /// Gets the number of players in the game
    fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Gets the guesses for the current round
    fn get_current_round_guesses(&self) -> Result<&HashMap<Player, String>, GameError> {
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
        let player = Player {
            id: 1,
            name: "Alice".to_string(),
        };

        game.add_player(player.clone()).unwrap();

        assert_eq!(game.player_count(), 1);
        assert_eq!(game.get_state(), &GameState::WaitingForPlayers);
        assert_eq!(game.players[0], player);
    }

    #[test]
    fn can_start_game_when_two_players_added() {
        let mut game = Game::new(1);
        let player1 = Player {
            id: 1,
            name: "Alice".to_string(),
        };
        let player2 = Player {
            id: 2,
            name: "Bob".to_string(),
        };

        game.add_player(player1.clone()).unwrap();
        assert_eq!(game.get_state(), &GameState::WaitingForPlayers);

        game.add_player(player2.clone()).unwrap();

        assert_eq!(game.player_count(), 2);
        assert_eq!(game.get_state(), &GameState::InProgress);
        assert_eq!(game.players[0], player1);
        assert_eq!(game.players[1], player2);
    }

    #[test]
    fn cannot_add_third_player_to_full_game() {
        let mut game = Game::new(1);
        let player1 = Player {
            id: 1,
            name: "Alice".to_string(),
        };
        let player2 = Player {
            id: 2,
            name: "Bob".to_string(),
        };
        let player3 = Player {
            id: 3,
            name: "Charlie".to_string(),
        };

        game.add_player(player1).unwrap();
        game.add_player(player2).unwrap();
        assert_eq!(game.get_state(), &GameState::InProgress);

        // Adding a third player should return an error
        let result = game.add_player(player3);

        assert_eq!(result, Err(GameError::GameFull));
        assert_eq!(game.player_count(), 2);
        assert_eq!(game.get_state(), &GameState::InProgress);
    }

    #[test]
    fn cannot_add_player_to_game_with_two_players_waiting() {
        let mut game = Game::new(1);
        let player1 = Player {
            id: 1,
            name: "Alice".to_string(),
        };
        let player2 = Player {
            id: 2,
            name: "Bob".to_string(),
        };
        let player3 = Player {
            id: 3,
            name: "Charlie".to_string(),
        };

        // Manually start the game first to keep it in WaitingForPlayers state
        game.add_player(player1).unwrap();
        game.add_player(player2).unwrap();

        // Try to add third player - should fail
        let result = game.add_player(player3);

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
        let player1 = Player {
            id: 1,
            name: "Alice".to_string(),
        };
        let player2 = Player {
            id: 2,
            name: "Bob".to_string(),
        };

        game.add_player(player1.clone()).unwrap();
        game.add_player(player2.clone()).unwrap();
        game.start_round();

        // Submit different guesses first
        let result = game
            .submit_guess(player1.clone(), "apple".to_string())
            .unwrap();
        assert!(!result); // Game should not end yet
        assert_eq!(game.get_current_round_guesses().unwrap().len(), 1);
        assert_eq!(game.get_state(), &GameState::InProgress);

        let result = game
            .submit_guess(player2.clone(), "banana".to_string())
            .unwrap();
        assert!(!result); // Game should not end yet
        assert_eq!(game.get_current_round_guesses().unwrap().len(), 2);
        assert_eq!(game.get_state(), &GameState::InProgress);

        // Start a new round and have both players guess the same word
        game.start_round();
        let result1 = game
            .submit_guess(player1.clone(), "orange".to_string())
            .unwrap();
        assert!(!result1); // Game should not end yet

        let result2 = game
            .submit_guess(player2.clone(), "orange".to_string())
            .unwrap();
        assert!(result2); // Game should end now
        assert_eq!(game.get_state(), &GameState::Finished);
    }

    #[test]
    fn cannot_submit_guess_if_game_not_in_progress() {
        let mut game = Game::new(1);
        let player = Player {
            id: 1,
            name: "Alice".to_string(),
        };

        game.add_player(player.clone()).unwrap();

        // Game is not started, so submitting a guess should fail
        let result = game.submit_guess(player, "apple".to_string());

        assert_eq!(result, Err(GameError::GameNotInProgress));
    }

    #[test]
    fn cannot_submit_guess_if_player_not_in_game() {
        let mut game = Game::new(1);
        let player1 = Player {
            id: 1,
            name: "Alice".to_string(),
        };
        let player2 = Player {
            id: 2,
            name: "Bob".to_string(),
        };
        let player3 = Player {
            id: 3,
            name: "Charlie".to_string(),
        };

        game.add_player(player1.clone()).unwrap();
        game.add_player(player2.clone()).unwrap();

        game.start_round();

        // Submitting a guess for a player not in the game should return an error
        let result = game.submit_guess(player3, "apple".to_string());

        assert_eq!(result, Err(GameError::PlayerNotFound));
    }

    #[test]
    fn cannot_submit_guess_if_no_active_round() {
        let mut game = Game::new(1);
        let player = Player {
            id: 1,
            name: "Alice".to_string(),
        };

        game.add_player(player.clone()).unwrap();
        game.start_game();

        // If no round is active, submitting a guess should return an error
        let result = game.submit_guess(player, "apple".to_string());

        assert_eq!(result, Err(GameError::NoActiveRound));
    }

    #[test]
    fn ends_game_when_both_players_guess_same_word() {
        let mut game = Game::new(1);
        let player1 = Player {
            id: 1,
            name: "Alice".to_string(),
        };
        let player2 = Player {
            id: 2,
            name: "Bob".to_string(),
        };

        game.add_player(player1.clone()).unwrap();
        game.add_player(player2.clone()).unwrap();
        game.start_round();

        // First player guesses
        let result1 = game
            .submit_guess(player1.clone(), "apple".to_string())
            .unwrap();
        assert!(!result1); // Game should not end yet
        assert_eq!(game.get_state(), &GameState::InProgress);

        // Second player guesses the same word
        let result2 = game
            .submit_guess(player2.clone(), "apple".to_string())
            .unwrap();
        assert!(result2); // Game should end now
        assert_eq!(game.get_state(), &GameState::Finished);

        // Current round should be ended and moved to completed rounds
        assert!(game.current_round.is_none());
        assert_eq!(game.rounds.len(), 1);
    }

    #[test]
    fn does_not_end_game_when_players_guess_different_words() {
        let mut game = Game::new(1);
        let player1 = Player {
            id: 1,
            name: "Alice".to_string(),
        };
        let player2 = Player {
            id: 2,
            name: "Bob".to_string(),
        };

        game.add_player(player1.clone()).unwrap();
        game.add_player(player2.clone()).unwrap();
        game.start_round();

        // Players guess different words
        let result1 = game
            .submit_guess(player1.clone(), "apple".to_string())
            .unwrap();
        assert!(!result1); // Game should not end yet

        let result2 = game
            .submit_guess(player2.clone(), "banana".to_string())
            .unwrap();
        assert!(!result2); // Game should still not end
        assert_eq!(game.get_state(), &GameState::InProgress);
    }
}

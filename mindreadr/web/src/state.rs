//! Client-side persistent state helpers for Mindreadr.
//!
//! Stores per-game player assignments so the client remembers which player
//! identity (Player1/Player2) was allocated when reloading or returning.

use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const STORAGE_KEY: &str = "mindreadr_state";

/// Client-side persisted state for the Mindreadr application.
///
/// This stores per-game player identifiers assigned by the server / core logic
/// (e.g. "Player1", "Player2") so the client can remember which player it is
/// when interacting with an existing game.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MindreadrState {
    /// Mapping of game id -> player name assigned in that game.
    pub game_players: HashMap<u32, String>,
}

impl MindreadrState {
    /// Loads the state from LocalStorage, returning an empty default if none present
    /// or if deserialization fails.
    pub fn load() -> Self {
        LocalStorage::get::<MindreadrState>(STORAGE_KEY).unwrap_or_default()
    }

    /// Persists the current state snapshot into LocalStorage.
    pub fn save(&self) {
        let _ = LocalStorage::set(STORAGE_KEY, self);
    }

    /// Inserts / updates the player name for a game and immediately persists the change.
    pub fn set_player_for_game(&mut self, game_id: u32, player_name: String) {
        self.game_players.insert(game_id, player_name);
        self.save();
    }

    /// Gets the stored player name for a given game id, if any.
    pub fn player_for_game(&self, game_id: u32) -> Option<&str> {
        self.game_players.get(&game_id).map(|s| s.as_str())
    }
}

/// Convenience hook for accessing the global MindreadrState persisted in LocalStorage.
///
/// Returns a UsePersistent<MindreadrState> handle with helper update semantics.
pub fn use_mindreadr_state() -> UsePersistent<MindreadrState> {
    use_persistent("mindreadr_state", MindreadrState::default)
}

/// A persistent storage hook that can be used to store data across application reloads.
///
/// Contract:
/// - key: unique storage key
/// - init: supplier to construct value if no stored value
/// - persists immediately on set/update
pub fn use_persistent<T: Serialize + DeserializeOwned + Default + 'static>(
    key: impl ToString,
    init: impl FnOnce() -> T,
) -> UsePersistent<T> {
    let state = use_signal(move || {
        let key = key.to_string();
        let value = LocalStorage::get(key.as_str()).ok().unwrap_or_else(init);
        StorageEntry { key, value }
    });
    UsePersistent { inner: state }
}

#[derive(Debug)]
struct StorageEntry<T> {
    key: String,
    value: T,
}

/// Storage that persists across application reloads.
#[derive(Copy, Clone)]
pub struct UsePersistent<T: 'static> {
    inner: Signal<StorageEntry<T>>,
}

impl<T: Serialize + DeserializeOwned + Clone + 'static> UsePersistent<T> {
    /// Gets a cloned value snapshot.
    pub fn get(&self) -> T {
        self.inner.read().value.clone()
    }

    /// Replaces the stored value and persists.
    pub fn set(&mut self, value: T) {
        let mut inner = self.inner.write();
        let _ = LocalStorage::set(inner.key.as_str(), &value);
        inner.value = value;
    }

    /// Applies a closure to mutate the value in-place and persists afterwards.
    pub fn update(&mut self, f: impl FnOnce(&mut T)) {
        let mut inner = self.inner.write();
        f(&mut inner.value);
        let _ = LocalStorage::set(inner.key.as_str(), &inner.value);
    }
}

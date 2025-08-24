//! Client-side persistent state helpers for Mindreadr.
//!
//! Stores per-game player assignments so the client remembers which player
//! identity (Player1/Player2) was allocated when reloading or returning.

use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Storage key for the per-game player mapping.
const GAME_PLAYER_MAP_KEY: &str = "game_player_map";

/// Minimal client-side state holding only the mapping from game id to the
/// locally assigned player name (e.g. "Player1", "Player2").
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct GamePlayerMap {
    /// Map of game id -> player name.
    pub by_game: HashMap<u32, String>,
}

impl GamePlayerMap {
    /// Assigns (or overwrites) the player name for a game.
    pub fn assign(&mut self, game_id: u32, player_name: String) {
        self.by_game.insert(game_id, player_name);
    }

    /// Returns the stored player name for a game, if any.
    pub fn get(&self, game_id: u32) -> Option<&str> {
        self.by_game.get(&game_id).map(|s| s.as_str())
    }
}

/// Hook returning a persistent GamePlayerMap stored in LocalStorage.
pub fn use_game_player_map() -> UsePersistent<GamePlayerMap> {
    use_persistent(GAME_PLAYER_MAP_KEY, GamePlayerMap::default)
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

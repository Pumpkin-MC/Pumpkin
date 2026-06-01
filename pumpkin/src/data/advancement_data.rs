use crate::entity::player::Player;
use crate::entity::player::advancement::{AdvancementDataError, PlayerAdvancement};
use pumpkin_data::Advancement;
use pumpkin_util::identifier::Identifier;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display};
use std::fs::create_dir_all;
use std::hash::Hash;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use tracing::{error, info};
use uuid::Uuid;

/// Manages player advancements, including data creation and saving.
pub struct AdvancementManager {
    pub advancement_path: PathBuf,
    pub save_enabled: bool,
}

impl AdvancementManager {
    /// Creates a new instance of `AdvancementManager` using the player data path.
    pub fn new(player_data_path: impl Into<PathBuf>, save_enabled: bool) -> Self {
        let path = player_data_path.into().join("advancements");
        if !path.exists()
            && let Err(e) = create_dir_all(&path)
        {
            error!(
                "Failed to create player data directory at {}: {e}",
                path.display()
            );
        }
        Self {
            advancement_path: path,
            save_enabled,
        }
    }

    /// Retrieves the list of all available advancements in the game.
    #[must_use]
    #[inline]
    pub const fn get_advancements(&self) -> Vec<Identifier> {
        Advancement::get_list().to_vec()
    }

    /// Creates and returns a new instance of `PlayerAdvancement` with the configured path.
    #[inline]
    #[must_use]
    pub const fn new_player_advancement(self: Arc<Self>, owner: Uuid) -> PlayerAdvancement {
        PlayerAdvancement::new(self, owner)
    }

    /// Saves the advancements of all provided players.
    pub async fn save_all_players(players: Vec<Arc<Player>>) -> Result<(), AdvancementDataError> {
        for player in players {
            player.advancements.lock().await.save()?;
        }
        Ok(())
    }

    /// Saves the advancements of a specific player.
    pub async fn save_player(player: &Player) -> Result<(), AdvancementDataError> {
        player.advancements.lock().await.save()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn advancement_manager_new() {
        let path = PathBuf::from("test_data");
        let manager = AdvancementManager::new(path, true);
        assert_eq!(
            manager.advancement_path,
            PathBuf::from("test_data/advancements")
        );
    }

    #[test]
    fn get_advancement_path() {
        let path = PathBuf::from("world/playerdata");
        let manager = AdvancementManager::new(path, true);
        let advancement_path = manager.advancement_path;
        assert!(advancement_path.ends_with("advancements"));
    }
}
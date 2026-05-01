use crate::entity::player::Player;
use crate::entity::player::advancement::{AdvancementDataError, PlayerAdvancement};
use pumpkin_data::Advancement;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::error;
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
    pub fn get_advancements(&self) -> Vec<&'static str> {
        Advancement::get_list().to_vec()
    }

    /// Creates and returns a new instance of `PlayerAdvancement` with the configured path.
    pub fn new_advancement(self: Arc<Self>, owner: Uuid) -> PlayerAdvancement {
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
    fn test_advancement_manager_new() {
        let path = PathBuf::from("test_data");
        let manager = AdvancementManager::new(path, true);
        assert_eq!(
            manager.advancement_path,
            PathBuf::from("test_data/advancements")
        );
    }

    #[test]
    fn test_get_advancement_path() {
        let path = PathBuf::from("world/playerdata");
        let manager = AdvancementManager::new(path, true);
        let advancement_path = manager.advancement_path;
        assert!(advancement_path.ends_with("advancements"));
    }
}

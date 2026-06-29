use crate::entity::player::Player;
use crate::entity::player::advancement::{AdvancementDataError, PlayerAdvancement};
use pumpkin_data::Advancement;
use pumpkin_util::identifier::Identifier;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::slice;
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
    #[must_use]
    #[inline]
    pub fn get_advancements(&self) -> Vec<Identifier> {
        Advancement::get_identifier_list().to_vec()
    }

    /// Creates and returns a new instance of `PlayerAdvancement` with the configured path.
    #[inline]
    #[must_use]
    pub fn new_player_advancement(self: Arc<Self>, owner: Uuid) -> PlayerAdvancement {
        PlayerAdvancement::new(self, owner)
    }

    /// Saves the advancements of all provided players.
    pub async fn save_all_players(
        &self,
        players: &[Arc<Player>],
    ) -> Result<(), AdvancementDataError> {
        if !self.save_enabled {
            return Ok(());
        }
        let mut to_write = Vec::with_capacity(players.len());
        for player in players {
            let guard = player.advancements.lock().await;
            let json = serde_json::to_string_pretty(&*guard).map_err(AdvancementDataError::Json)?;
            to_write.push((guard.path.clone(), json));
        }

        let advancement_path = self.advancement_path.clone();
        tokio::task::spawn_blocking(move || {
            if let Err(e) = create_dir_all(advancement_path) {
                error!("Failed to create player advancement directory : {e}");
                return Err(AdvancementDataError::Io(e));
            }
            for (path, json) in to_write {
                std::fs::write(&path, json).map_err(AdvancementDataError::Io)?;
            }
            Ok(())
        })
        .await
        .expect("spawn_blocking task panicked")
    }

    /// Saves the advancements of a specific player.
    pub async fn save_player(&self, player: &Arc<Player>) -> Result<(), AdvancementDataError> {
        self.save_all_players(slice::from_ref(player)).await
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

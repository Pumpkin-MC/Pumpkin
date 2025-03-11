use crate::{
    entity::{NBTStorage, player::Player},
    server::Server,
};
use crossbeam::atomic::AtomicCell;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_world::data::player_data::{PlayerDataError, PlayerDataStorage};
use std::sync::Arc;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
/// Helper for managing player data in the server context.
///
/// This struct provides server-wide access to the `PlayerDataStorage` and
/// convenience methods for player handling.
pub struct ServerPlayerData {
    storage: Arc<PlayerDataStorage>,
    save_interval: Duration,
    last_save: AtomicCell<Instant>,
}

impl ServerPlayerData {
    /// Creates a new `ServerPlayerData` with specified configuration.
    pub fn new(data_path: impl Into<PathBuf>, save_interval: Duration) -> Self {
        Self {
            storage: Arc::new(PlayerDataStorage::new(data_path)),
            save_interval,
            last_save: AtomicCell::new(Instant::now()),
        }
    }

    /// Handles a player joining the server.
    ///
    /// This function loads player data and applies it to a newly joined player.
    ///
    /// # Arguments
    ///
    /// * `player` - The player who joined.
    ///
    /// # Returns
    ///
    /// A Result indicating success or the error that occurred.
    pub async fn handle_player_join(&self, player: &mut Player) -> Result<(), PlayerDataError> {
        self.load_and_apply_data_to_player(player).await
    }

    /// Handles a player leaving the server.
    ///
    /// This function saves player data when they disconnect.
    ///
    /// # Arguments
    ///
    /// * `player` - The player who left.
    ///
    /// # Returns
    ///
    /// A Result indicating success or the error that occurred.
    pub async fn handle_player_leave(&self, player: &Player) -> Result<(), PlayerDataError> {
        let mut nbt = NbtCompound::new();
        player.write_nbt(&mut nbt).await;

        // Save to disk
        self.storage
            .save_player_data(&player.gameprofile.id, nbt)
            .await?;

        Ok(())
    }

    /// Performs periodic maintenance tasks.
    ///
    /// This function should be called regularly to save player data and clean
    /// expired cache entries.
    pub async fn tick(&self, server: &Server) -> Result<(), PlayerDataError> {
        let now = Instant::now();

        // Only save players periodically based on save_interval
        let last_save = self.last_save.load();
        let should_save = now.duration_since(last_save) >= self.save_interval;

        if should_save && self.storage.save_enabled {
            self.last_save.store(now);
            // Save all online players periodically across all worlds
            for world in server.worlds.read().await.iter() {
                let players = world.players.read().await;
                for player in players.values() {
                    let mut nbt = NbtCompound::new();
                    player.write_nbt(&mut nbt).await;

                    // Save to disk periodically to prevent data loss on server crash
                    if let Err(e) = self
                        .storage
                        .save_player_data(&player.gameprofile.id, nbt)
                        .await
                    {
                        log::error!(
                            "Failed to save player data for {}: {}",
                            player.gameprofile.id,
                            e
                        );
                    }
                }
            }

            log::debug!("Periodic player data save completed");
        }

        Ok(())
    }

    /// Saves all players' data immediately.
    ///
    /// This function immediately saves all online players' data to disk.
    /// Useful for server shutdown or backup operations.
    pub async fn save_all_players(&self, server: &Server) -> Result<(), PlayerDataError> {
        let mut total_players = 0;

        // Save players from all worlds
        for world in server.worlds.read().await.iter() {
            let players = world.players.read().await;
            for player in players.values() {
                self.extract_data_and_save_player(player).await?;
                total_players += 1;
            }
        }

        log::debug!("Saved data for {} online players", total_players);
        Ok(())
    }

    /// Loads player data and applies it to a player.
    ///
    /// This function loads a player's data and applies it to their Player instance.
    /// For new players, it creates default data without errors.
    ///
    /// # Arguments
    ///
    /// * `player` - The player to load data for and apply to.
    ///
    /// # Returns
    ///
    /// A Result indicating success or the error that occurred.
    pub async fn load_and_apply_data_to_player(
        &self,
        player: &mut Player,
    ) -> Result<(), PlayerDataError> {
        let uuid = &player.gameprofile.id;
        match self.storage.load_player_data(uuid).await {
            Ok(mut data) => {
                player.read_nbt(&mut data).await;
                Ok(())
            }
            Err(e) => {
                if self.storage.save_enabled {
                    // Only log as error if player data saving is enabled
                    log::error!("Error loading player data for {}: {}", uuid, e);
                } else {
                    // Otherwise just log as info since it's expected
                    log::debug!("Not loading player data for {} (saving disabled)", uuid);
                }
                // Continue with default data even if there's an error
                Ok(())
            }
        }
    }

    /// Extracts and saves data from a player.
    ///
    /// This function extracts NBT data from a player and saves it to disk.
    ///
    /// # Arguments
    ///
    /// * `player` - The player to extract and save data for.
    ///
    /// # Returns
    ///
    /// A Result indicating success or the error that occurred.
    pub async fn extract_data_and_save_player(
        &self,
        player: &Player,
    ) -> Result<(), PlayerDataError> {
        let uuid = &player.gameprofile.id;
        let mut nbt = NbtCompound::new();
        player.write_nbt(&mut nbt).await;
        self.storage.save_player_data(uuid, nbt).await
    }
}

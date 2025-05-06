use crate::{
    entity::{NBTStorage, player::Player},
    server::Server,
};
use crossbeam::atomic::AtomicCell;
use pumpkin_inventory::screen_handler::ScreenHandler;
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
        player
            .player_screen_handler
            .lock()
            .await
            .on_closed(player)
            .await;
        player.on_handled_screen_closed().await;

        let mut nbt = NbtCompound::new();
        player.write_nbt(&mut nbt).await;

        // Save to disk
        self.storage.save_player_data(&player.gameprofile.id, nbt)?;

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

        if should_save && self.storage.is_save_enabled() {
            self.last_save.store(now);
            // Save all online players periodically across all worlds
            for world in server.worlds.read().await.iter() {
                for player in world.players.read().await.values() {
                    let mut nbt = NbtCompound::new();
                    player.write_nbt(&mut nbt).await;

                    // Save to disk periodically to prevent data loss on server crash
                    if let Err(e) = self.storage.save_player_data(&player.gameprofile.id, nbt) {
                        log::error!(
                            "Failed to save player data for {}: {e}",
                            player.gameprofile.id,
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
            for player in world.players.read().await.values() {
                self.extract_data_and_save_player(player).await?;
                total_players += 1;
            }
        }

        log::debug!("Saved data for {total_players} online players");
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
        match self.storage.load_player_data(uuid) {
            Ok((should_load, mut data)) => {
                if !should_load {
                    // No data to load, continue with default data
                    return Ok(());
                }
                player.read_nbt(&mut data).await;
                Ok(())
            }
            Err(e) => {
                if self.storage.is_save_enabled() {
                    // Only log as error if player data saving is enabled
                    log::error!("Error loading player data for {uuid}: {e}");
                } else {
                    // Otherwise just log as info since it's expected
                    log::debug!("Not loading player data for {uuid} (saving disabled)");
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
        if !self.storage.is_save_enabled() {
            return Ok(());
        }

        let uuid = &player.gameprofile.id;
        let mut nbt = NbtCompound::new();
        player.write_nbt(&mut nbt).await;
        self.storage.save_player_data(uuid, nbt)
    }
}

#[cfg(test)]
mod test {
    use crate::data::player_server_data::ServerPlayerData;
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_world::data::player_data::PlayerDataStorage;
    use std::time::Duration;
    use std::time::Instant;
    use tempfile::tempdir;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_player_data_storage_new() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let storage = PlayerDataStorage::new(path.clone());

        assert_eq!(storage.get_data_path().as_path(), path.as_path());
        // Note: save_enabled might be configured differently in your actual code
    }

    #[tokio::test]
    async fn test_player_data_storage_get_player_data_path() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let storage = PlayerDataStorage::new(path.clone());

        let uuid = Uuid::new_v4();
        let expected_path = path.join(format!("{uuid}.dat"));

        assert_eq!(storage.get_player_data_path(&uuid), expected_path);
    }

    #[tokio::test]
    async fn test_player_data_storage_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let mut storage = PlayerDataStorage::new(path);
        storage.set_save_enabled(true); // Ensure saving is enabled for this test

        let uuid = Uuid::new_v4();

        // Create test data
        let mut nbt = NbtCompound::new();
        nbt.put_string("TestKey", "TestValue".to_string());
        nbt.put_int("TestInt", 42);

        // Save the data
        storage.save_player_data(&uuid, nbt).unwrap();

        // Load the data
        let (load_success, loaded_nbt) = storage.load_player_data(&uuid).unwrap();

        assert!(load_success);
        assert_eq!(loaded_nbt.get_string("TestKey").unwrap(), "TestValue");
        assert_eq!(loaded_nbt.get_int("TestInt").unwrap(), 42);
    }

    #[tokio::test]
    async fn test_player_data_storage_load_nonexistent() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let mut storage = PlayerDataStorage::new(path);
        storage.set_save_enabled(true); // Ensure saving is enabled for this test

        let uuid = Uuid::new_v4();

        // Try to load non-existent data
        let (load_success, empty_nbt) = storage.load_player_data(&uuid).unwrap();

        assert!(!load_success);
        assert_eq!(empty_nbt.child_tags.len(), 0);
    }

    #[tokio::test]
    async fn test_player_data_storage_disabled() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let mut storage = PlayerDataStorage::new(path);
        storage.set_save_enabled(false);

        let uuid = Uuid::new_v4();
        let mut nbt = NbtCompound::new();
        nbt.put_string("TestKey", "TestValue".to_string());

        // Save should succeed but do nothing
        let save_result = storage.save_player_data(&uuid, nbt);
        assert!(save_result.is_ok());

        // Load should return empty data
        let (load_success, empty_nbt) = storage.load_player_data(&uuid).unwrap();
        assert!(!load_success);
        assert_eq!(empty_nbt.child_tags.len(), 0);
    }

    #[tokio::test]
    async fn test_server_player_data_new() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();
        let save_interval = Duration::from_secs(300);

        let player_data = ServerPlayerData::new(path, save_interval);

        assert_eq!(player_data.save_interval, save_interval);
        assert!(
            Instant::now().duration_since(player_data.last_save.load()) < Duration::from_secs(1)
        );
    }

    #[tokio::test]
    async fn test_player_data_file_structure() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let uuid = Uuid::new_v4();
        let mut storage = PlayerDataStorage::new(path.clone());
        storage.set_save_enabled(true);

        // Create and save player data
        let mut nbt = NbtCompound::new();
        nbt.put_string("name", "TestPlayer".to_string());
        nbt.put_int("level", 42);
        storage.save_player_data(&uuid, nbt).unwrap();

        // Verify the file exists
        let player_data_path = storage.get_player_data_path(&uuid);
        assert!(player_data_path.exists());

        // Load it again and verify content
        let (success, loaded_data) = storage.load_player_data(&uuid).unwrap();
        assert!(success);
        assert_eq!(loaded_data.get_string("name").unwrap(), "TestPlayer");
        assert_eq!(loaded_data.get_int("level").unwrap(), 42);
    }
}

use crate::{
    entity::{NBTStorage, player::Player},
    server::Server,
};
use crossbeam::atomic::AtomicCell;
use pumpkin_inventory::screen_handler::ScreenHandler;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_world::data::player_data::{PlayerDataError, PlayerDataStorage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tracing::{debug, error};
use uuid::Uuid;

/// Helper for managing player data in the server context.
///
/// This struct provides server-wide access to the `PlayerDataStorage` and
/// convenience methods for player handling.
pub struct ServerPlayerData {
    storage: Arc<PlayerDataStorage>,
    save_interval: Duration,
    last_save: AtomicCell<Instant>,
    /// Serializes player-data writes and orders them per player.
    ///
    /// Every accepted save bumps the player's epoch in this map, and a
    /// queued periodic snapshot is only written back while its epoch is
    /// still current. This keeps a delayed periodic save from
    /// overwriting a newer save of the same player (e.g. the disconnect
    /// save).
    write_epochs: Arc<Mutex<HashMap<Uuid, u64>>>,
}

impl ServerPlayerData {
    /// Creates a new `ServerPlayerData` with specified configuration.
    pub fn new(data_path: impl Into<PathBuf>, save_interval: Duration, enabled: bool) -> Self {
        Self {
            storage: Arc::new(PlayerDataStorage::new(data_path, enabled)),
            save_interval,
            last_save: AtomicCell::new(Instant::now()),
            write_epochs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Saves `nbt` for `uuid` immediately, ordering the write against any
    /// queued periodic save of the same player.
    fn save_player_data_ordered(
        &self,
        uuid: &Uuid,
        nbt: NbtCompound,
    ) -> Result<(), PlayerDataError> {
        let mut epochs = self
            .write_epochs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *epochs.entry(*uuid).or_insert(0) += 1;
        self.storage.save_player_data(uuid, nbt)
    }

    /// Writes back a queued periodic snapshot only if no newer save of the
    /// same player superseded it while it was queued. The write happens
    /// while holding the epoch lock, so it is serialized against every
    /// other save of this player.
    fn save_periodic_snapshot(
        storage: &PlayerDataStorage,
        write_epochs: &Mutex<HashMap<Uuid, u64>>,
        uuid: &Uuid,
        nbt: NbtCompound,
        epoch: u64,
    ) -> Result<(), PlayerDataError> {
        let epochs = write_epochs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if epochs.get(uuid).copied().unwrap_or(0) != epoch {
            return Ok(());
        }
        storage.save_player_data(uuid, nbt)
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
    pub fn handle_player_leave(&self, player: &Arc<Player>) -> Result<(), PlayerDataError> {
        player
            .player_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .on_closed(player.as_ref());
        player.on_handled_screen_closed();

        let mut nbt = NbtCompound::new();
        player.write_nbt(&mut nbt);

        self.save_player_data_ordered(&player.gameprofile.id, nbt)?;
        Ok(())
    }

    /// Performs periodic maintenance tasks.
    ///
    /// This function is called synchronously on the server tick loop to check
    /// if it is time to save player data.
    pub fn tick(&self, server: &Server) {
        let now = Instant::now();

        // Only save players periodically based on save_interval
        let last_save = self.last_save.load();
        let should_save = now.duration_since(last_save) >= self.save_interval;

        if should_save && self.storage.is_save_enabled() {
            self.last_save.store(now);
            // Snapshot all online players periodically across all worlds,
            // bumping each player's write epoch so a queued snapshot can
            // tell whether it is still the newest save when it runs.
            let mut snapshots = Vec::new();
            {
                let mut epochs = self
                    .write_epochs
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for world in server.worlds.load().iter() {
                    for player in world.players.load().iter() {
                        let mut nbt = NbtCompound::new();
                        player.write_nbt(&mut nbt);
                        let epoch = epochs.entry(player.gameprofile.id).or_insert(0);
                        *epoch += 1;
                        snapshots.push((player.gameprofile.id, nbt, *epoch));
                    }
                }
            }

            if snapshots.is_empty() {
                return;
            }

            let storage = self.storage.clone();
            let write_epochs = self.write_epochs.clone();
            rayon::spawn(move || {
                for (uuid, nbt, epoch) in snapshots {
                    if let Err(e) =
                        Self::save_periodic_snapshot(&storage, &write_epochs, &uuid, nbt, epoch)
                    {
                        error!("Failed to save player data for {uuid}: {e}");
                    }
                }
                debug!("Periodic player data save completed");
            });
        }
    }

    /// Saves all players' data immediately.
    ///
    /// This function immediately saves all online players' data to disk.
    /// Useful for server shutdown or backup operations.
    pub fn save_all_players(&self, server: &Server) -> Result<(), PlayerDataError> {
        let mut total_players = 0;

        // Save players from all worlds
        for world in server.worlds.load().iter() {
            for player in world.players.load().iter() {
                self.extract_data_and_save_player(player)?;
                total_players += 1;
            }
        }

        debug!("Saved data for {total_players} online players");
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
    pub fn load_data(&self, uuid: &uuid::Uuid) -> Result<Option<NbtCompound>, PlayerDataError> {
        let result = self.storage.load_player_data(uuid);

        match result {
            Ok((should_load, data)) => {
                if !should_load {
                    // No data to load, continue with default data
                    return Ok(None);
                }
                Ok(Some(data))
            }
            Err(e) => {
                if self.storage.is_save_enabled() {
                    // Only log as error if player data saving is enabled
                    error!("Error loading player data for {uuid}: {e}");
                } else {
                    // Otherwise just log as info since it's expected
                    debug!("Not loading player data for {uuid} (saving disabled)");
                }
                // Continue with default data even if there's an error
                Ok(None)
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
    pub fn extract_data_and_save_player(&self, player: &Player) -> Result<(), PlayerDataError> {
        if !self.storage.is_save_enabled() {
            return Ok(());
        }

        let uuid = player.gameprofile.id;
        let mut nbt = NbtCompound::new();
        player.write_nbt(&mut nbt);

        self.save_player_data_ordered(&uuid, nbt)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::data::player_server::ServerPlayerData;
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_world::data::player_data::PlayerDataStorage;
    use std::time::Duration;
    use std::time::Instant;
    use tempfile::tempdir;
    use uuid::Uuid;

    #[tokio::test]
    async fn player_data_storage_new() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let storage = PlayerDataStorage::new(path.clone(), true);

        assert_eq!(storage.get_data_path().as_path(), path.as_path());
        // Note: save_enabled might be configured differently in your actual code
    }

    #[tokio::test]
    async fn player_data_storage_get_player_data_path() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let storage = PlayerDataStorage::new(path.clone(), true);

        let uuid = Uuid::new_v4();
        let expected_path = path.join(format!("{uuid}.dat"));

        assert_eq!(storage.get_player_data_path(&uuid), expected_path);
    }

    #[test]
    fn stale_periodic_snapshot_does_not_overwrite_newer_save() {
        use std::collections::HashMap;
        use std::sync::{Arc, Mutex};

        let temp_dir = tempdir().unwrap();
        let storage = PlayerDataStorage::new(temp_dir.path().to_path_buf(), true);
        let write_epochs: Arc<Mutex<HashMap<Uuid, u64>>> = Arc::new(Mutex::new(HashMap::new()));

        let uuid = Uuid::new_v4();

        // Periodic tick snapshots the player at state A (epoch 1).
        let mut epochs = write_epochs.lock().unwrap();
        *epochs.entry(uuid).or_insert(0) += 1;
        let snapshot_epoch = *epochs.get(&uuid).unwrap();
        drop(epochs);
        let mut stale_nbt = NbtCompound::new();
        stale_nbt.put_string("TestKey", "StaleValue".to_string());

        // The player disconnects before the queued task runs; the
        // disconnect save writes the newer state B (epoch 2).
        {
            let mut epochs = write_epochs.lock().unwrap();
            *epochs.entry(uuid).or_insert(0) += 1;
        }
        let mut newer_nbt = NbtCompound::new();
        newer_nbt.put_string("TestKey", "NewerValue".to_string());
        storage.save_player_data(&uuid, newer_nbt).unwrap();

        // The queued periodic task must skip its stale snapshot.
        ServerPlayerData::save_periodic_snapshot(
            &storage,
            &write_epochs,
            &uuid,
            stale_nbt,
            snapshot_epoch,
        )
        .unwrap();

        let (loaded, nbt) = storage.load_player_data(&uuid).unwrap();
        assert!(loaded);
        assert_eq!(nbt.get_string("TestKey").unwrap(), "NewerValue");
    }

    #[tokio::test]
    async fn player_data_storage_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let storage = PlayerDataStorage::new(path, true); // Ensure saving is enabled for this test

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
    async fn player_data_storage_load_nonexistent() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let storage = PlayerDataStorage::new(path, true); // Ensure saving is enabled for this test

        let uuid = Uuid::new_v4();

        // Try to load non-existent data
        let (load_success, empty_nbt) = storage.load_player_data(&uuid).unwrap();

        assert!(!load_success);
        assert_eq!(empty_nbt.child_tags.len(), 0);
    }

    #[tokio::test]
    async fn player_data_storage_disabled() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let storage = PlayerDataStorage::new(path, false);

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
    async fn server_player_data_new() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();
        let save_interval = Duration::from_mins(5);

        let player_data = ServerPlayerData::new(path, save_interval, true);

        assert_eq!(player_data.save_interval, save_interval);
        assert!(
            Instant::now().duration_since(player_data.last_save.load()) < Duration::from_secs(1)
        );
    }

    #[tokio::test]
    async fn player_data_file_structure() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let uuid = Uuid::new_v4();
        let storage = PlayerDataStorage::new(path, true);

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

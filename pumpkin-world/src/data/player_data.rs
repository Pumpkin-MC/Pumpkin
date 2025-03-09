use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_nbt::compound::NbtCompound;
use std::fs::{File, create_dir_all};
use std::io;
use std::path::PathBuf;
use uuid::Uuid;

/// Manages the storage and retrieval of player data from disk and memory cache.
///
/// This struct provides functions to load and save player data to/from NBT files,
/// with a memory cache to handle player disconnections temporarily.
pub struct PlayerDataStorage {
    /// Path to the directory where player data is stored
    data_path: PathBuf,
    /// Whether player data saving is enabled
    pub save_enabled: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum PlayerDataError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("NBT error: {0}")]
    Nbt(String),
}

impl PlayerDataStorage {
    /// Creates a new `PlayerDataStorage` with the specified data path and cache expiration time.
    pub fn new(data_path: impl Into<PathBuf>) -> Self {
        let path = data_path.into();
        if !path.exists() {
            if let Err(e) = create_dir_all(&path) {
                log::error!(
                    "Failed to create player data directory at {:?}: {}",
                    path,
                    e
                );
            }
        }

        let config = &ADVANCED_CONFIG.player_data;

        Self {
            data_path: path,
            save_enabled: config.save_player_data,
        }
    }

    /// Returns the path for a player's data file based on their UUID.
    fn get_player_data_path(&self, uuid: &Uuid) -> PathBuf {
        self.data_path.join(format!("{uuid}.dat"))
    }

    /// Loads player data from NBT file or cache.
    ///
    /// This function first checks if player data exists in the cache.
    /// If not, it attempts to load the data from a .dat file on disk.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID of the player to load data for.
    ///
    /// # Returns
    ///
    /// A Result containing either the player's NBT data or an error.
    pub async fn load_player_data(&self, uuid: &Uuid) -> Result<NbtCompound, PlayerDataError> {
        // If player data saving is disabled, return empty data
        if !self.save_enabled {
            return Ok(NbtCompound::new());
        }

        // If not in cache, load from disk
        let path = self.get_player_data_path(uuid);
        if !path.exists() {
            log::debug!("No player data file found for {}", uuid);
            return Ok(NbtCompound::new());
        }

        // Offload file I/O to a separate tokio task
        let uuid_copy = *uuid;
        let nbt = tokio::task::spawn_blocking(move || -> Result<NbtCompound, PlayerDataError> {
            match File::open(&path) {
                Ok(file) => {
                    // Read directly from the file with GZip decompression
                    pumpkin_nbt::nbt_compress::read_gzip_compound_tag(file)
                        .map_err(|e| PlayerDataError::Nbt(e.to_string()))
                }
                Err(e) => Err(PlayerDataError::Io(e)),
            }
        })
        .await
        .unwrap_or_else(|e| {
            log::error!(
                "Task error when loading player data for {}: {}",
                uuid_copy,
                e
            );
            Err(PlayerDataError::Nbt(format!("Task join error: {e}")))
        })?;

        log::debug!("Loaded player data for {} from disk", uuid);
        Ok(nbt)
    }

    /// Saves player data to NBT file and updates cache.
    ///
    /// This function saves the player's data to a .dat file on disk and also
    /// updates the in-memory cache with the latest data.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID of the player to save data for.
    /// * `data` - The NBT compound data to save.
    ///
    /// # Returns
    ///
    /// A Result indicating success or the error that occurred.
    pub async fn save_player_data(
        &self,
        uuid: &Uuid,
        data: NbtCompound,
    ) -> Result<(), PlayerDataError> {
        // Skip saving if disabled in config
        if !self.save_enabled {
            return Ok(());
        }

        let path = self.get_player_data_path(uuid);

        // Run disk I/O in a separate tokio task
        let uuid_copy = *uuid;
        let data_clone = data;

        match tokio::spawn(async move {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                if let Err(e) = create_dir_all(parent) {
                    log::error!(
                        "Failed to create player data directory for {}: {}",
                        uuid_copy,
                        e
                    );
                    return Err(PlayerDataError::Io(e));
                }
            }

            // Create the file and write directly with GZip compression
            match File::create(&path) {
                Ok(file) => {
                    if let Err(e) =
                        pumpkin_nbt::nbt_compress::write_gzip_compound_tag(&data_clone, file)
                    {
                        log::error!(
                            "Failed to write compressed player data for {}: {}",
                            uuid_copy,
                            e
                        );
                        Err(PlayerDataError::Nbt(e.to_string()))
                    } else {
                        log::debug!("Saved player data for {} to disk", uuid_copy);
                        Ok(())
                    }
                }
                Err(e) => {
                    log::error!("Failed to create player data file for {}: {}", uuid_copy, e);
                    Err(PlayerDataError::Io(e))
                }
            }
        })
        .await
        {
            Ok(result) => result,
            Err(e) => {
                log::error!("Task panicked while saving player data for {}: {}", uuid, e);
                Err(PlayerDataError::Nbt(format!("Task join error: {e}")))
            }
        }
    }
}

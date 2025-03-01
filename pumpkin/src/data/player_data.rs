use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io,
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_nbt::compound::NbtCompound;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    entity::{NBTStorage, player::Player},
    server::Server,
};

/// Manages the storage and retrieval of player data from disk and memory cache.
///
/// This struct provides functions to load and save player data to/from NBT files,
/// with a memory cache to handle player disconnections temporarily.
pub struct PlayerDataStorage {
    /// Path to the directory where player data is stored
    data_path: PathBuf,
    /// In-memory cache of recently disconnected players' data
    cache: Mutex<HashMap<Uuid, (NbtCompound, Instant)>>,
    /// How long to keep player data in cache after disconnection
    cache_expiration: Duration,
    /// Maximum number of entries in the cache
    max_cache_entries: usize,
    /// Whether player data saving is enabled
    save_enabled: bool,
    /// Whether to cache player data
    cache_enabled: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum PlayerDataError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("NBT error: {0}")]
    Nbt(String),
    #[error("Player data not found for UUID: {0}")]
    NotFound(Uuid),
}

impl PlayerDataStorage {
    /// Creates a new `PlayerDataStorage` with the specified data path and cache expiration time.
    pub fn new(data_path: impl Into<PathBuf>, cache_expiration: Duration) -> Self {
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
            cache: Mutex::new(HashMap::new()),
            cache_expiration,
            max_cache_entries: config.max_cache_entries as usize,
            save_enabled: config.save_player_data,
            cache_enabled: config.cache_player_data,
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

        // Check cache first if caching is enabled
        if self.cache_enabled {
            let cache = self.cache.lock().await;
            if let Some((data, _)) = cache.get(uuid) {
                log::debug!(
                    "Loaded player data for {} from cache with data {:?}",
                    uuid,
                    data
                );
                return Ok(data.clone());
            }
        }

        // If not in cache, load from disk
        let path = self.get_player_data_path(uuid);
        if !path.exists() {
            log::debug!("No player data file found for {}", uuid);
            return Err(PlayerDataError::NotFound(*uuid));
        }

        // Offload file I/O to a separate tokio task
        let uuid_copy = *uuid;
        let nbt = tokio::task::spawn_blocking(move || -> Result<NbtCompound, PlayerDataError> {
            let mut file = File::open(&path).map_err(PlayerDataError::Io)?;
            let mut data = Vec::new();
            file.read_to_end(&mut data).map_err(PlayerDataError::Io)?;

            pumpkin_nbt::nbt_compress::read_gzip_compound_tag(&data)
                .map_err(|e| PlayerDataError::Nbt(e.to_string()))
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

        // Update cache if caching is enabled
        if self.cache_enabled {
            let mut cache = self.cache.lock().await;
            cache.insert(*uuid, (data.clone(), Instant::now()));
        };

        // Run disk I/O in a separate tokio task
        let uuid_copy = *uuid;
        tokio::spawn(async move {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                if let Err(e) = create_dir_all(parent) {
                    log::error!(
                        "Failed to create player data directory for {}: {}",
                        uuid_copy,
                        e
                    );
                    return;
                }
            }

            // Compress the NBT data
            let compressed = match pumpkin_nbt::nbt_compress::write_gzip_compound_tag(&data) {
                Ok(compressed) => compressed,
                Err(e) => {
                    log::error!("Failed to compress player data for {}: {}", uuid_copy, e);
                    return;
                }
            };

            // Save to disk
            match File::create(&path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(&compressed) {
                        log::error!("Failed to write player data for {}: {}", uuid_copy, e);
                    } else {
                        log::debug!("Saved player data for {} to disk", uuid_copy);
                    }
                }
                Err(e) => {
                    log::error!("Failed to create player data file for {}: {}", uuid_copy, e);
                }
            }
        });
        Ok(())
    }

    /// Caches player data on disconnect to avoid loading from disk on rejoin.
    ///
    /// This function is used when a player disconnects, to temporarily cache their
    /// data in memory with an expiration timestamp.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID of the player who disconnected.
    /// * `data` - The NBT compound data to cache.
    pub async fn cache_on_disconnect(&self, uuid: &Uuid, data: NbtCompound) {
        // Skip if caching is disabled
        if !self.cache_enabled {
            return;
        }

        // Clone the data to avoid holding locks during complex operations
        let data_clone = data.clone();

        // Use a scope to limit the lock duration
        {
            let mut cache = self.cache.lock().await;

            // Check if we need to remove an entry to stay under max_cache_entries
            if cache.len() >= self.max_cache_entries && !cache.contains_key(uuid) {
                // Find the oldest entry
                if let Some(oldest_uuid) = cache
                    .iter()
                    .min_by_key(|(_, (_, timestamp))| *timestamp)
                    .map(|(uuid, _)| *uuid)
                {
                    cache.remove(&oldest_uuid);
                }
            }

            // Insert the new entry
            cache.insert(*uuid, (data_clone, Instant::now()));
        };

        log::debug!("Cached player data for {} on disconnect", uuid);
    }

    /// Removes expired player data from the cache.
    ///
    /// This function should be called periodically to clean up cached player data
    /// that has exceeded its expiration time.
    pub async fn clean_expired_cache(&self) {
        if !self.cache_enabled {
            return;
        }

        let mut cache = self.cache.lock().await;
        let now = Instant::now();
        let expired: Vec<Uuid> = cache
            .iter()
            .filter(|(_, (_, timestamp))| now.duration_since(*timestamp) > self.cache_expiration)
            .map(|(uuid, _)| *uuid)
            .collect();

        for uuid in expired {
            cache.remove(&uuid);
        }

        // Release lock before waiting for tasks
        drop(cache);
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
        match self.load_player_data(uuid).await {
            Ok(mut data) => {
                player.read_nbt(&mut data).await;
                Ok(())
            }
            Err(PlayerDataError::NotFound(_)) => {
                // For new players, just continue with default data
                log::debug!("Creating new player data for {}", uuid);
                Ok(())
            }
            Err(e) => {
                if self.save_enabled {
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
        self.save_player_data(uuid, nbt).await
    }
}

/// Helper for managing player data in the server context.
///
/// This struct provides server-wide access to the `PlayerDataStorage` and
/// convenience methods for player handling.
pub struct ServerPlayerData {
    storage: Arc<PlayerDataStorage>,
    save_interval: Duration,
    last_cleanup: Mutex<Instant>,
    cleanup_interval: Duration,
    last_save: Mutex<Instant>,
}

impl ServerPlayerData {
    /// Creates a new `ServerPlayerData` with specified configuration.
    pub fn new(
        data_path: impl Into<PathBuf>,
        cache_expiration: Duration,
        save_interval: Duration,
        cleanup_interval: Duration,
    ) -> Self {
        Self {
            storage: Arc::new(PlayerDataStorage::new(data_path, cache_expiration)),
            save_interval,
            last_cleanup: Mutex::new(Instant::now()),
            cleanup_interval,
            last_save: Mutex::new(Instant::now()),
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
        self.storage.load_and_apply_data_to_player(player).await
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

        // First cache it if caching is enabled
        self.storage
            .cache_on_disconnect(&player.gameprofile.id, nbt.clone())
            .await;

        // Then save to disk
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

        // Check if cleanup is needed
        {
            let mut last_cleanup = self.last_cleanup.lock().await;
            if now.duration_since(*last_cleanup) >= self.cleanup_interval {
                self.storage.clean_expired_cache().await;
                *last_cleanup = now;
            }
        }

        // Only save players periodically based on save_interval
        let should_save = {
            let mut last_save = self.last_save.lock().await;
            let should_save = now.duration_since(*last_save) >= self.save_interval;

            if should_save {
                *last_save = now;
            }

            should_save
        };

        if should_save && self.storage.save_enabled {
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
                self.storage.extract_data_and_save_player(player).await?;
                total_players += 1;
            }
        }

        log::debug!("Saved data for {} online players", total_players);
        Ok(())
    }
}

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::translation::localized_log_format;
use std::fs::{File, create_dir_all};
use std::io;
use std::path::PathBuf;
use tracing::{debug, error};
use uuid::Uuid;

/// Manages the storage and retrieval of player data from disk and memory cache.
///
/// This struct provides functions to load and save player data to/from NBT files,
/// with a memory cache to handle player disconnections temporarily.
pub struct PlayerDataStorage {
    /// Path to the directory where player data is stored
    data_path: PathBuf,
    /// Whether player data saving is enabled
    save_enabled: bool,
}

#[derive(Debug)]
pub enum PlayerDataError {
    Io(io::Error),
    Nbt(String),
}

impl std::fmt::Display for PlayerDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(
                f,
                "{}",
                localized_log_format("world.player_data.error.io", &[e.to_string()])
            ),
            Self::Nbt(msg) => write!(
                f,
                "{}",
                localized_log_format("world.player_data.error.nbt", &[msg.clone()])
            ),
        }
    }
}

impl std::error::Error for PlayerDataError {}

impl From<io::Error> for PlayerDataError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl PlayerDataStorage {
    /// Creates a new `PlayerDataStorage` with the specified data path and cache expiration time.
    pub fn new(data_path: impl Into<PathBuf>, enabled: bool) -> Self {
        let path = data_path.into();
        if !path.exists()
            && let Err(e) = create_dir_all(&path)
        {
            error!(
                "{}",
                localized_log_format(
                    "world.player_data.create_dir_failed",
                    &[path.display().to_string(), e.to_string()]
                )
            );
        }

        Self {
            data_path: path,
            save_enabled: enabled,
        }
    }

    #[must_use]
    pub const fn get_data_path(&self) -> &PathBuf {
        &self.data_path
    }

    #[must_use]
    pub const fn is_save_enabled(&self) -> bool {
        self.save_enabled
    }

    pub const fn set_save_enabled(&mut self, enabled: bool) {
        self.save_enabled = enabled;
    }

    /// Returns the path for a player's data file based on their UUID.
    #[must_use]
    pub fn get_player_data_path(&self, uuid: &Uuid) -> PathBuf {
        self.get_data_path().join(format!("{uuid}.dat"))
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
    pub fn load_player_data(&self, uuid: &Uuid) -> Result<(bool, NbtCompound), PlayerDataError> {
        // If player data saving is disabled, return empty data
        if !self.is_save_enabled() {
            return Ok((false, NbtCompound::new()));
        }

        // If not in cache, load from disk
        let path = self.get_player_data_path(uuid);
        if !path.exists() {
            debug!(
                "{}",
                localized_log_format("world.player_data.no_file", &[uuid.to_string()])
            );
            return Ok((false, NbtCompound::new()));
        }

        let file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                error!(
                    "{}",
                    localized_log_format(
                        "world.player_data.open_failed",
                        &[uuid.to_string(), e.to_string()]
                    )
                );
                return Err(PlayerDataError::Io(e));
            }
        };

        match pumpkin_nbt::nbt_compress::read_gzip_compound_tag(file) {
            Ok(nbt) => {
                debug!(
                    "{}",
                    localized_log_format("world.player_data.loaded", &[uuid.to_string()])
                );
                Ok((true, nbt))
            }
            Err(e) => {
                error!(
                    "{}",
                    localized_log_format(
                        "world.player_data.read_failed",
                        &[uuid.to_string(), e.to_string()]
                    )
                );
                Err(PlayerDataError::Nbt(e.to_string()))
            }
        }
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
    pub fn save_player_data(&self, uuid: &Uuid, data: NbtCompound) -> Result<(), PlayerDataError> {
        // Skip saving if disabled in config
        if !self.is_save_enabled() {
            return Ok(());
        }

        let path = self.get_player_data_path(uuid);

        // Ensure parent directory exists
        if let Some(parent) = path.parent()
            && let Err(e) = create_dir_all(parent)
        {
            error!(
                "{}",
                localized_log_format(
                    "world.player_data.create_directory_failed",
                    &[uuid.to_string(), e.to_string()]
                )
            );
            return Err(PlayerDataError::Io(e));
        }

        // Create the file and write directly with GZip compression
        match File::create(&path) {
            Ok(file) => {
                if let Err(e) = pumpkin_nbt::nbt_compress::write_gzip_compound_tag(data, file) {
                    error!(
                        "{}",
                        localized_log_format(
                            "world.player_data.write_compressed_failed",
                            &[uuid.to_string(), e.to_string()]
                        )
                    );
                    Err(PlayerDataError::Nbt(e.to_string()))
                } else {
                    debug!(
                        "{}",
                        localized_log_format("world.player_data.saved", &[uuid.to_string()])
                    );
                    Ok(())
                }
            }
            Err(e) => {
                error!(
                    "{}",
                    localized_log_format(
                        "world.player_data.create_file_failed",
                        &[uuid.to_string(), e.to_string()]
                    )
                );
                Err(PlayerDataError::Io(e))
            }
        }
    }
}

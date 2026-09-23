use pumpkin_nbt::compound::NbtCompound;
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

#[derive(Debug, thiserror::Error)]
pub enum PlayerDataError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("NBT error: {0}")]
    Nbt(String),
}

impl PlayerDataStorage {
    /// Creates a new `PlayerDataStorage` with the specified data path and cache expiration time.
    pub fn new(data_path: impl Into<PathBuf>, enabled: bool) -> Self {
        let path = data_path.into();
        if !path.exists()
            && let Err(e) = create_dir_all(&path)
        {
            error!(
                "Failed to create player data directory at {}: {e}",
                path.display()
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
            debug!("No player data file found for {uuid}");
            return Ok((false, NbtCompound::new()));
        }

        let file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open player data file for {uuid}: {e}");
                return Err(PlayerDataError::Io(e));
            }
        };

        match pumpkin_nbt::nbt_compress::read_gzip_compound_tag(file) {
            Ok(nbt) => {
                debug!("Loaded player data for {uuid} from disk");
                Ok((true, nbt))
            }
            Err(e) => {
                error!("Failed to read player data for {uuid}: {e}");
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
            error!("Failed to create player data directory for {uuid}: {e}");
            return Err(PlayerDataError::Io(e));
        }

        Self::write_atomically(&path, |file| {
            pumpkin_nbt::nbt_compress::write_gzip_compound_tag(data, file)
                .map_err(|e| PlayerDataError::Nbt(e.to_string()))
        })
        .map_err(|e| {
            error!("Failed to write compressed player data for {uuid}: {e}");
            e
        })?;

        debug!("Saved player data for {uuid} to disk");
        Ok(())
    }

    fn write_atomically(
        path: &std::path::Path,
        write: impl FnOnce(&File) -> Result<(), PlayerDataError>,
    ) -> Result<(), PlayerDataError> {
        static TEMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

        let unique = TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temp_path = path.with_extension(format!("dat.tmp.{}.{unique}", std::process::id()));

        let result = (|| {
            let file = File::create(&temp_path)?;
            write(&file)?;
            file.sync_all()?;
            std::fs::rename(&temp_path, path)?;
            Ok(())
        })();

        if result.is_err() {
            let _ = std::fs::remove_file(&temp_path);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_storage() -> (tempfile::TempDir, PlayerDataStorage) {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = PlayerDataStorage::new(temp_dir.path(), true);
        (temp_dir, storage)
    }

    fn test_compound(value: &str) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.put_string("State", value.to_string());
        nbt
    }

    #[test]
    fn save_and_load_round_trip() {
        let (_dir, storage) = temp_storage();
        let uuid = Uuid::new_v4();

        storage
            .save_player_data(&uuid, test_compound("good"))
            .unwrap();

        let (found, loaded) = storage.load_player_data(&uuid).unwrap();
        assert!(found);
        assert_eq!(loaded.get_string("State").unwrap(), "good");
    }

    #[test]
    fn failed_write_leaves_previous_file_intact() {
        let (dir, storage) = temp_storage();
        let uuid = Uuid::new_v4();
        storage
            .save_player_data(&uuid, test_compound("good"))
            .unwrap();

        let path = storage.get_player_data_path(&uuid);
        let result = PlayerDataStorage::write_atomically(&path, |mut file| {
            file.write_all(b"partial garbage").unwrap();
            Err(PlayerDataError::Nbt("injected failure".to_string()))
        });
        assert!(result.is_err());

        let (found, loaded) = storage.load_player_data(&uuid).unwrap();
        assert!(found);
        assert_eq!(loaded.get_string("State").unwrap(), "good");

        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp."))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn no_temp_files_left_after_save() {
        let (dir, storage) = temp_storage();
        let uuid = Uuid::new_v4();

        storage
            .save_player_data(&uuid, test_compound("good"))
            .unwrap();
        storage
            .save_player_data(&uuid, test_compound("better"))
            .unwrap();

        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(entries, vec![format!("{uuid}.dat")]);
    }
}

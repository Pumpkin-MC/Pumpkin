//! Vanilla-compatible `level.dat` — gzipped NBT at the world root. A
//! `level.dat_old` backup is written on each successful load.

use std::io::{Cursor, Read};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Deserialize;
use tokio::fs;

use crate::BoxFuture;
use crate::error::StorageError;
use crate::vanilla::VanillaStorage;
use crate::world_info::{
    LevelData, MAXIMUM_SUPPORTED_LEVEL_VERSION, MAXIMUM_SUPPORTED_WORLD_DATA_VERSION,
    MINIMUM_SUPPORTED_LEVEL_VERSION, MINIMUM_SUPPORTED_WORLD_DATA_VERSION, WorldInfoStorage,
};

pub const LEVEL_DAT_FILE_NAME: &str = "level.dat";
pub const LEVEL_DAT_BACKUP_FILE_NAME: &str = "level.dat_old";

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LevelDat {
    #[serde(rename = "Data")]
    data: LevelData,
}

fn check_data_version(raw_nbt: &[u8]) -> Result<(), StorageError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct DataVersionOnly {
        data_version: i32,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Wrapper {
        data: DataVersionOnly,
    }

    let info: Wrapper = pumpkin_nbt::from_bytes(Cursor::new(raw_nbt))
        .map_err(|e| StorageError::Deserialize(e.to_string()))?;
    let v = info.data.data_version;
    if (MINIMUM_SUPPORTED_WORLD_DATA_VERSION..=MAXIMUM_SUPPORTED_WORLD_DATA_VERSION).contains(&v) {
        Ok(())
    } else {
        Err(StorageError::UnsupportedVersion(format!(
            "world data version {v} out of supported range"
        )))
    }
}

fn check_level_version(raw_nbt: &[u8]) -> Result<(), StorageError> {
    #[derive(Deserialize)]
    struct LevelVersionOnly {
        version: i32,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Wrapper {
        data: LevelVersionOnly,
    }

    let info: Wrapper = pumpkin_nbt::from_bytes(Cursor::new(raw_nbt))
        .map_err(|e| StorageError::Deserialize(e.to_string()))?;
    let v = info.data.version;
    if (MINIMUM_SUPPORTED_LEVEL_VERSION..=MAXIMUM_SUPPORTED_LEVEL_VERSION).contains(&v) {
        Ok(())
    } else {
        Err(StorageError::UnsupportedVersion(format!(
            "level version {v} out of supported range"
        )))
    }
}

impl WorldInfoStorage for VanillaStorage {
    fn load(&self) -> BoxFuture<'_, Result<LevelData, StorageError>> {
        Box::pin(async move {
            let path = self.world_dir().join(LEVEL_DAT_FILE_NAME);
            let compressed = fs::read(&path).await.map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    StorageError::NotFound {
                        message: format!("level.dat not found at {}", path.display()),
                    }
                } else {
                    StorageError::io_at(&path, e)
                }
            })?;

            let mut buf = Vec::new();
            GzDecoder::new(Cursor::new(compressed))
                .read_to_end(&mut buf)
                .map_err(StorageError::io)?;

            check_data_version(&buf)?;
            check_level_version(&buf)?;

            let dat: LevelDat = pumpkin_nbt::from_bytes(Cursor::new(buf))
                .map_err(|e| StorageError::Deserialize(e.to_string()))?;

            let backup = self.world_dir().join(LEVEL_DAT_BACKUP_FILE_NAME);
            if let Err(e) = fs::copy(&path, &backup).await
                && e.kind() != std::io::ErrorKind::NotFound
            {
                return Err(StorageError::io_at(&backup, e));
            }

            Ok(dat.data)
        })
    }

    fn save<'a>(&'a self, data: &'a LevelData) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async move {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_millis() as i64);
            let mut stamped = data.clone();
            stamped.last_played = now_ms;
            let dat = LevelDat { data: stamped };

            let mut compressed = Vec::new();
            {
                let mut encoder = GzEncoder::new(&mut compressed, Compression::best());
                pumpkin_nbt::to_bytes(&dat, &mut encoder)
                    .map_err(|e| StorageError::Serialize(e.to_string()))?;
                encoder.finish().map_err(StorageError::io)?
            };

            let path = self.world_dir().join(LEVEL_DAT_FILE_NAME);
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| StorageError::io_at(parent, e))?;
            }
            fs::write(&path, &compressed)
                .await
                .map_err(|e| StorageError::io_at(&path, e))?;
            Ok(())
        })
    }
}

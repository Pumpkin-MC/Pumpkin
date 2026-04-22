//! Per-player PNBT (raw) under `<world_dir>/playerdata/<uuid>.dat`. Bytes
//! are written as-is — PNBT is pumpkin-internal and not vanilla-compatible.

use std::path::PathBuf;

use async_trait::async_trait;
use pumpkin_nbt::pnbt::PNbtCompound;
use tokio::fs;
use uuid::Uuid;

use crate::error::StorageError;
use crate::player_data::PlayerDataStorage;
use crate::vanilla::VanillaStorage;

const PLAYERDATA_DIR: &str = "playerdata";

impl VanillaStorage {
    fn player_data_dir(&self) -> PathBuf {
        self.world_dir().join(PLAYERDATA_DIR)
    }

    fn player_data_path(&self, uuid: Uuid) -> PathBuf {
        self.player_data_dir().join(format!("{uuid}.dat"))
    }
}

#[async_trait]
impl PlayerDataStorage for VanillaStorage {
    async fn load(&self, uuid: Uuid) -> Result<PNbtCompound, StorageError> {
        let path = self.player_data_path(uuid);
        let bytes = fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound {
                    message: format!("no player data for {uuid}"),
                }
            } else {
                StorageError::io_at(&path, e)
            }
        })?;
        Ok(PNbtCompound::from_bytes(bytes))
    }

    async fn save(&self, uuid: Uuid, data: &PNbtCompound) -> Result<(), StorageError> {
        let dir = self.player_data_dir();
        fs::create_dir_all(&dir)
            .await
            .map_err(|e| StorageError::io_at(&dir, e))?;

        let path = self.player_data_path(uuid);
        fs::write(&path, data.as_bytes())
            .await
            .map_err(|e| StorageError::io_at(&path, e))?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Uuid>, StorageError> {
        let dir = self.player_data_dir();
        let mut entries = match fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(StorageError::io_at(&dir, e)),
        };

        let mut ids = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StorageError::io_at(&dir, e))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("dat") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if let Ok(id) = Uuid::parse_str(stem) {
                ids.push(id);
            }
        }
        Ok(ids)
    }
}

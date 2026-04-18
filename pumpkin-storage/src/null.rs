//! Null (discarding) storage backend.
//!
//! Every `save` succeeds and drops the data. Every `load` returns
//! [`StorageError::NotFound`]. `list` returns an empty collection.
//!
//! Useful for test setups, disabled-persistence configurations, and as a
//! placeholder when a domain trait must be wired up but no real storage is
//! desired.

use async_trait::async_trait;
use pumpkin_nbt::compound::NbtCompound;
use uuid::Uuid;

use crate::error::StorageError;
use crate::level_info::{LevelData, LevelInfoStorage};
use crate::player_data::PlayerDataStorage;

#[derive(Debug, Default, Clone, Copy)]
pub struct NullStorage;

impl NullStorage {
    pub const fn new() -> Self {
        Self
    }
}

fn not_found(what: &str) -> StorageError {
    StorageError::NotFound {
        message: format!("null storage has no {what}"),
    }
}

#[async_trait]
impl LevelInfoStorage for NullStorage {
    async fn load(&self) -> Result<LevelData, StorageError> {
        Err(not_found("level info"))
    }

    async fn save(&self, _data: &LevelData) -> Result<(), StorageError> {
        Ok(())
    }
}

#[async_trait]
impl PlayerDataStorage for NullStorage {
    async fn load(&self, uuid: Uuid) -> Result<NbtCompound, StorageError> {
        Err(not_found(&format!("player data for {uuid}")))
    }

    async fn save(&self, _uuid: Uuid, _data: &NbtCompound) -> Result<(), StorageError> {
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Uuid>, StorageError> {
        Ok(Vec::new())
    }
}

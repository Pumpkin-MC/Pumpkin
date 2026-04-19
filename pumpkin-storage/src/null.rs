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
use time::OffsetDateTime;
use uuid::Uuid;

use std::net::IpAddr;

use pumpkin_config::op::Op;
use pumpkin_util::permission::PermissionLvl;

use crate::banlist::{BannedIpEntry, BannedPlayerEntry};
use crate::banned_ip::BannedIpStorage;
use crate::banned_player::BannedPlayerStorage;
use crate::error::StorageError;
use crate::level_info::{LevelData, LevelInfoStorage};
use crate::op::OpStorage;
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

#[async_trait]
impl OpStorage for NullStorage {
    async fn op(
        &self,
        _uuid: Uuid,
        _name: &str,
        _level: PermissionLvl,
        _bypasses_player_limit: bool,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn deop(&self, _uuid: Uuid) -> Result<(), StorageError> {
        Ok(())
    }

    async fn is_op(&self, _uuid: Uuid) -> Result<bool, StorageError> {
        Ok(false)
    }

    async fn get(&self, _uuid: Uuid) -> Result<Option<Op>, StorageError> {
        Ok(None)
    }

    async fn list(&self) -> Result<Vec<Op>, StorageError> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl BannedIpStorage for NullStorage {
    async fn ban(
        &self,
        _ip: IpAddr,
        _source: String,
        _expires: Option<OffsetDateTime>,
        _reason: String,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn unban(&self, _ip: IpAddr) -> Result<(), StorageError> {
        Ok(())
    }

    async fn is_banned(&self, _ip: IpAddr) -> Result<bool, StorageError> {
        Ok(false)
    }

    async fn get(&self, _ip: IpAddr) -> Result<Option<BannedIpEntry>, StorageError> {
        Ok(None)
    }

    async fn list(&self) -> Result<Vec<BannedIpEntry>, StorageError> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl BannedPlayerStorage for NullStorage {
    async fn ban(
        &self,
        _uuid: Uuid,
        _name: &str,
        _source: String,
        _expires: Option<OffsetDateTime>,
        _reason: String,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn unban(&self, _uuid: Uuid) -> Result<(), StorageError> {
        Ok(())
    }

    async fn is_banned(&self, _uuid: Uuid) -> Result<bool, StorageError> {
        Ok(false)
    }

    async fn get(&self, _uuid: Uuid) -> Result<Option<BannedPlayerEntry>, StorageError> {
        Ok(None)
    }

    async fn list(&self) -> Result<Vec<BannedPlayerEntry>, StorageError> {
        Ok(Vec::new())
    }
}

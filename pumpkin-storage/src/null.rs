//! Null (discarding) storage backend.
//!
//! Every `save` succeeds and drops the data. Every `load` returns
//! [`StorageError::NotFound`]. `list` returns an empty collection.
//!
//! Useful for test setups, disabled-persistence configurations, and as a
//! placeholder when a domain trait must be wired up but no real storage is
//! desired.

use pumpkin_nbt::pnbt::PNbtCompound;
use time::OffsetDateTime;
use uuid::Uuid;

use std::net::IpAddr;

use pumpkin_config::op::Op;
use pumpkin_config::whitelist::WhitelistEntry;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::permission::PermissionLvl;
use tokio::sync::mpsc;

use crate::BoxFuture;
use crate::banlist::{BannedIpEntry, BannedPlayerEntry};
use crate::banned_ip::BannedIpStorage;
use crate::banned_player::BannedPlayerStorage;
use crate::chunk::{ChunkStorage, LoadedData};
use crate::error::StorageError;
use crate::level_info::{LevelData, LevelInfoStorage};
use crate::op::OpStorage;
use crate::player_data::PlayerDataStorage;
use crate::poi::PoiStorage;
use crate::user_cache::{UserCacheEntry, UserCacheStorage};
use crate::whitelist::WhitelistStorage;

#[derive(Debug, Default, Clone, Copy)]
pub struct NullStorage;

impl NullStorage {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

fn not_found(what: &str) -> StorageError {
    StorageError::NotFound {
        message: format!("null storage has no {what}"),
    }
}

impl LevelInfoStorage for NullStorage {
    fn load(&self) -> BoxFuture<'_, Result<LevelData, StorageError>> {
        Box::pin(async { Err(not_found("level info")) })
    }

    fn save<'a>(&'a self, _data: &'a LevelData) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }
}

impl PlayerDataStorage for NullStorage {
    fn load(&self, uuid: Uuid) -> BoxFuture<'_, Result<PNbtCompound, StorageError>> {
        Box::pin(async move { Err(not_found(&format!("player data for {uuid}"))) })
    }

    fn save<'a>(
        &'a self,
        _uuid: Uuid,
        _data: &'a PNbtCompound,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<Uuid>, StorageError>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}

impl<T: Send + Sync + 'static> ChunkStorage<T> for NullStorage {
    fn fetch_chunks<'a>(
        &'a self,
        chunk_coords: &'a [Vector2<i32>],
        stream: mpsc::Sender<LoadedData<T>>,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            for coord in chunk_coords {
                if stream.send(LoadedData::Missing(*coord)).await.is_err() {
                    break;
                }
            }
        })
    }

    fn save_chunks(
        &self,
        _chunks: Vec<(Vector2<i32>, T)>,
    ) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn watch_chunks<'a>(&'a self, _chunks: &'a [Vector2<i32>]) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn unwatch_chunks<'a>(&'a self, _chunks: &'a [Vector2<i32>]) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn clear_watched_chunks(&self) -> BoxFuture<'_, ()> {
        Box::pin(async {})
    }
    fn block_and_await_ongoing_tasks(&self) -> BoxFuture<'_, ()> {
        Box::pin(async {})
    }
}

impl PoiStorage for NullStorage {
    fn add<'a>(
        &'a self,
        _pos: BlockPos,
        _poi_type: &'a str,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn remove(&self, _pos: BlockPos) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async { Ok(false) })
    }

    fn get_in_square<'a>(
        &'a self,
        _center: BlockPos,
        _radius: i32,
        _poi_type: Option<&'a str>,
    ) -> BoxFuture<'a, Result<Vec<BlockPos>, StorageError>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn save_all(&self) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }
}

impl UserCacheStorage for NullStorage {
    fn upsert<'a>(
        &'a self,
        _uuid: Uuid,
        _name: &'a str,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn get_by_uuid(
        &self,
        _uuid: Uuid,
    ) -> BoxFuture<'_, Result<Option<UserCacheEntry>, StorageError>> {
        Box::pin(async { Ok(None) })
    }

    fn get_by_name<'a>(
        &'a self,
        _name: &'a str,
    ) -> BoxFuture<'a, Result<Option<UserCacheEntry>, StorageError>> {
        Box::pin(async { Ok(None) })
    }
}

impl WhitelistStorage for NullStorage {
    fn add<'a>(&'a self, _uuid: Uuid, _name: &'a str) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn remove(&self, _uuid: Uuid) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn is_whitelisted(&self, _uuid: Uuid) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async { Ok(false) })
    }

    fn get(&self, _uuid: Uuid) -> BoxFuture<'_, Result<Option<WhitelistEntry>, StorageError>> {
        Box::pin(async { Ok(None) })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<WhitelistEntry>, StorageError>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn reload(&self) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }
}

impl OpStorage for NullStorage {
    fn op<'a>(
        &'a self,
        _uuid: Uuid,
        _name: &'a str,
        _level: PermissionLvl,
        _bypasses_player_limit: bool,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn deop(&self, _uuid: Uuid) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn is_op(&self, _uuid: Uuid) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async { Ok(false) })
    }

    fn get(&self, _uuid: Uuid) -> BoxFuture<'_, Result<Option<Op>, StorageError>> {
        Box::pin(async { Ok(None) })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<Op>, StorageError>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}

impl BannedIpStorage for NullStorage {
    fn ban(
        &self,
        _ip: IpAddr,
        _source: String,
        _expires: Option<OffsetDateTime>,
        _reason: String,
    ) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn unban(&self, _ip: IpAddr) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn is_banned(&self, _ip: IpAddr) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async { Ok(false) })
    }

    fn get(&self, _ip: IpAddr) -> BoxFuture<'_, Result<Option<BannedIpEntry>, StorageError>> {
        Box::pin(async { Ok(None) })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<BannedIpEntry>, StorageError>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}

impl BannedPlayerStorage for NullStorage {
    fn ban<'a>(
        &'a self,
        _uuid: Uuid,
        _name: &'a str,
        _source: String,
        _expires: Option<OffsetDateTime>,
        _reason: String,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn unban(&self, _uuid: Uuid) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }

    fn is_banned(&self, _uuid: Uuid) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async { Ok(false) })
    }

    fn get(&self, _uuid: Uuid) -> BoxFuture<'_, Result<Option<BannedPlayerEntry>, StorageError>> {
        Box::pin(async { Ok(None) })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<BannedPlayerEntry>, StorageError>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}

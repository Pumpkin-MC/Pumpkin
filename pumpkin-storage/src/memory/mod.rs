use std::collections::HashMap;

use pumpkin_config::op::Op;
use pumpkin_config::whitelist::WhitelistEntry;
use pumpkin_nbt::pnbt::PNbtCompound;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::banlist::{BannedIpEntry, BannedPlayerEntry};
use crate::level_info::LevelData;
use crate::poi::PoiEntry;
use crate::user_cache::UserCacheEntry;

mod banned_ip;
mod banned_player;
pub mod chunk;
mod level_info;
mod op;
mod player_data;
mod poi;
mod user_cache;
mod whitelist;

pub use chunk::MemoryChunkStorage;

/// Format-agnostic, in-memory storage.
///
/// Unlike [`VanillaStorage`](crate::VanillaStorage), this backend holds domain
/// values directly (no serialization, no on-disk layout). Intended for tests,
/// ephemeral servers, and embedded contexts where persistence is not needed.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    pub(crate) level_info: RwLock<Option<LevelData>>,
    pub(crate) player_data: RwLock<HashMap<Uuid, PNbtCompound>>,
    pub(crate) banned_players: RwLock<Vec<BannedPlayerEntry>>,
    pub(crate) banned_ips: RwLock<Vec<BannedIpEntry>>,
    pub(crate) ops: RwLock<Vec<Op>>,
    pub(crate) whitelist: RwLock<Vec<WhitelistEntry>>,
    pub(crate) user_cache: RwLock<std::collections::HashMap<Uuid, UserCacheEntry>>,
    pub(crate) poi: RwLock<std::collections::HashMap<(i32, i32, i32), PoiEntry>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

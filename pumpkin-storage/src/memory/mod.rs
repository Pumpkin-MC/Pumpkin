use std::collections::HashMap;

use pumpkin_nbt::compound::NbtCompound;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::banlist::{BannedIpEntry, BannedPlayerEntry};
use crate::level_info::LevelData;

mod banned_ip;
mod banned_player;
mod level_info;
mod player_data;

/// Format-agnostic, in-memory storage.
///
/// Unlike [`VanillaStorage`](crate::VanillaStorage), this backend holds domain
/// values directly (no serialization, no on-disk layout). Intended for tests,
/// ephemeral servers, and embedded contexts where persistence is not needed.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    pub(crate) level_info: RwLock<Option<LevelData>>,
    pub(crate) player_data: RwLock<HashMap<Uuid, NbtCompound>>,
    pub(crate) banned_players: RwLock<Vec<BannedPlayerEntry>>,
    pub(crate) banned_ips: RwLock<Vec<BannedIpEntry>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

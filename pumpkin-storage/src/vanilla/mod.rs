use std::path::{Path, PathBuf};

use tokio::sync::RwLock;

use pumpkin_config::op::Op;
use pumpkin_config::whitelist::WhitelistEntry;

use crate::banlist::{BannedIpEntry, BannedPlayerEntry};

mod banned_ip;
mod banned_player;
mod level_info;
mod op;
mod player_data;
mod poi;
mod user_cache;
mod whitelist;

use poi::PoiInner;
use user_cache::UserCacheInner;

#[allow(unused_imports)]
pub use level_info::{LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME};

/// Filesystem-backed storage laid out the way vanilla Minecraft expects.
///
/// Vanilla keeps two distinct roots:
/// - `world_dir`: the world folder (`level.dat`, `playerdata/`, `region/`,
///   `poi/`, ...). Moves with the world and is per-world.
/// - `server_data_dir`: the server-wide `data/` folder (`ops.json`,
///   `whitelist.json`, `banned-players.json`, `banned-ips.json`,
///   `usercache.json`). Shared across worlds on the same server.
///
/// Domain traits implemented on `VanillaStorage` translate their operations
/// into file I/O under the appropriate root.
#[derive(Debug)]
pub struct VanillaStorage {
    world_dir: PathBuf,
    server_data_dir: PathBuf,
    pub(crate) banned_players: RwLock<Option<Vec<BannedPlayerEntry>>>,
    pub(crate) banned_ips: RwLock<Option<Vec<BannedIpEntry>>>,
    pub(crate) ops: RwLock<Option<Vec<Op>>>,
    pub(crate) whitelist: RwLock<Option<Vec<WhitelistEntry>>>,
    pub(crate) user_cache_inner: tokio::sync::Mutex<UserCacheInner>,
    pub(crate) poi_inner: tokio::sync::Mutex<PoiInner>,
}

impl VanillaStorage {
    pub fn new(world_dir: impl Into<PathBuf>, server_data_dir: impl Into<PathBuf>) -> Self {
        Self {
            world_dir: world_dir.into(),
            server_data_dir: server_data_dir.into(),
            banned_players: RwLock::new(None),
            banned_ips: RwLock::new(None),
            ops: RwLock::new(None),
            whitelist: RwLock::new(None),
            user_cache_inner: tokio::sync::Mutex::new(UserCacheInner::default()),
            poi_inner: tokio::sync::Mutex::new(PoiInner::default()),
        }
    }

    pub fn world_dir(&self) -> &Path {
        &self.world_dir
    }

    pub fn server_data_dir(&self) -> &Path {
        &self.server_data_dir
    }
}

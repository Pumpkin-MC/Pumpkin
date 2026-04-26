//! Points of Interest storage.

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use serde::{Deserialize, Serialize};

use crate::BoxFuture;
use crate::error::StorageError;

pub const POI_TYPE_NETHER_PORTAL: &str = "minecraft:nether_portal";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoiEntry {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    #[serde(rename = "type")]
    pub poi_type: String,
    pub free_tickets: i32,
}

impl PoiEntry {
    #[must_use]
    pub const fn pos(&self) -> BlockPos {
        BlockPos(Vector3::new(self.x, self.y, self.z))
    }
}

pub trait PoiStorage: Send + Sync {
    fn add<'a>(
        &'a self,
        pos: BlockPos,
        poi_type: &'a str,
    ) -> BoxFuture<'a, Result<(), StorageError>>;

    fn remove(&self, pos: BlockPos) -> BoxFuture<'_, Result<bool, StorageError>>;

    /// Returns every POI whose Chebyshev distance (max of |dx|, |dz|) from
    /// `center` is `<= radius`, optionally filtered by `poi_type`.
    fn get_in_square<'a>(
        &'a self,
        center: BlockPos,
        radius: i32,
        poi_type: Option<&'a str>,
    ) -> BoxFuture<'a, Result<Vec<BlockPos>, StorageError>>;

    /// Flushes any in-memory state to persistent storage.
    fn save_all(&self) -> BoxFuture<'_, Result<(), StorageError>>;
}

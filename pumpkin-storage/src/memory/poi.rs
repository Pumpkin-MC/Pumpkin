//! In-memory [`PoiStorage`] keyed by `(x, y, z)`. No persistence —
//! `save_all` is a no-op.

use async_trait::async_trait;
use pumpkin_util::math::position::BlockPos;

use crate::error::StorageError;
use crate::memory::MemoryStorage;
use crate::poi::{PoiEntry, PoiStorage};

#[async_trait]
impl PoiStorage for MemoryStorage {
    async fn add(&self, pos: BlockPos, poi_type: &str) -> Result<(), StorageError> {
        self.poi.write().await.insert(
            (pos.0.x, pos.0.y, pos.0.z),
            PoiEntry {
                x: pos.0.x,
                y: pos.0.y,
                z: pos.0.z,
                poi_type: poi_type.to_string(),
                free_tickets: 0,
            },
        );
        Ok(())
    }

    async fn remove(&self, pos: BlockPos) -> Result<bool, StorageError> {
        Ok(self
            .poi
            .write()
            .await
            .remove(&(pos.0.x, pos.0.y, pos.0.z))
            .is_some())
    }

    async fn get_in_square(
        &self,
        center: BlockPos,
        radius: i32,
        poi_type: Option<&str>,
    ) -> Result<Vec<BlockPos>, StorageError> {
        Ok(self
            .poi
            .read()
            .await
            .values()
            .filter(|entry| {
                poi_type.is_none_or(|t| entry.poi_type == t)
                    && (entry.x - center.0.x).abs() <= radius
                    && (entry.z - center.0.z).abs() <= radius
            })
            .map(|entry| entry.pos())
            .collect())
    }

    async fn save_all(&self) -> Result<(), StorageError> {
        Ok(())
    }
}

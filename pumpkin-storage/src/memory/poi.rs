//! In-memory [`PoiStorage`] keyed by `(x, y, z)`. No persistence —
//! `save_all` is a no-op.

use pumpkin_util::math::position::BlockPos;

use crate::BoxFuture;
use crate::error::StorageError;
use crate::memory::MemoryStorage;
use crate::poi::{PoiEntry, PoiStorage};

impl PoiStorage for MemoryStorage {
    fn add<'a>(
        &'a self,
        pos: BlockPos,
        poi_type: &'a str,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async move {
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
        })
    }

    fn remove(&self, pos: BlockPos) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async move {
            Ok(self
                .poi
                .write()
                .await
                .remove(&(pos.0.x, pos.0.y, pos.0.z))
                .is_some())
        })
    }

    fn get_in_square<'a>(
        &'a self,
        center: BlockPos,
        radius: i32,
        poi_type: Option<&'a str>,
    ) -> BoxFuture<'a, Result<Vec<BlockPos>, StorageError>> {
        Box::pin(async move {
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
                .map(PoiEntry::pos)
                .collect())
        })
    }

    fn save_all(&self) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }
}

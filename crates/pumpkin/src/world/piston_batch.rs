//! Sequential tick island for a moving piston construction.
//!
//! Vanilla `Level.tickBlockEntities` / `entityTickList` is single-threaded creation order.
//! Concurrent BE/entity ticks would race on which placeholder places first. Placeholders
//! (`is_tick_order_sensitive`) tick here one at a time; overlapping entities join that sequence.

use std::sync::Arc;

use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};

use crate::block::entities::BlockEntity;
use crate::world::World;

/// One cell of slack around a moving block (`PistonMath.getMovementArea` sweeps a full cell).
///
/// The destination occupant is pushed, so it must tick in this batch, not in the entity `JoinSet`.
const CONSTRUCTION_REACH: f64 = 1.0;

struct Entry {
    /// Creation sequence from `World::block_entity_tick_order`. `u64::MAX` if restored from NBT.
    order: u64,
    position: BlockPos,
    block_entity: Arc<dyn BlockEntity>,
}

/// Order-sensitive BEs of one tick. Gathered before the entity phase so `overlaps` can pull
/// colliding entities out of the concurrent `JoinSet`.
#[derive(Default)]
pub struct PistonBatch {
    entries: Vec<Entry>,
    areas: Vec<BoundingBox>,
}

impl PistonBatch {
    /// Adds one order-sensitive BE and the volume it occupies (cell plus `CONSTRUCTION_REACH`).
    pub fn push(&mut self, order: Option<u64>, block_entity: Arc<dyn BlockEntity>) {
        let position = block_entity.get_position();
        self.areas
            .push(BoundingBox::from_block(&position).expand_all(CONSTRUCTION_REACH));
        self.entries.push(Entry {
            order: order.unwrap_or(u64::MAX),
            position,
            block_entity,
        });
    }

    /// Vanilla `blockEntityTickers` order: creation sequence, oldest first.
    ///
    /// Position (y, z, x) is the tiebreak for NBT-restored BEs with no sequence number, so the
    /// order is total and stable across ticks.
    pub fn sort(&mut self) {
        self.entries.sort_unstable_by_key(|entry| {
            (
                entry.order,
                entry.position.0.y,
                entry.position.0.z,
                entry.position.0.x,
            )
        });
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// True if this entity must tick sequentially with the construction, not in the `JoinSet`.
    ///
    /// A cart on a piston-driven slime reads the rail the placeholders rewrite this tick; a
    /// concurrent tick would race that write.
    #[must_use]
    pub fn overlaps(&self, bounding_box: &BoundingBox) -> bool {
        self.areas.iter().any(|area| area.intersects(bounding_box))
    }

    /// Ticks placeholders one at a time, in the order `sort` established.
    ///
    /// After the concurrent BE `JoinSet`. Sequential `await`: two placeholders cannot place
    /// into the same neighbour on the same tick.
    pub async fn tick_block_entities(&self, world: &Arc<World>) {
        for entry in &self.entries {
            if world
                .get_block_state_id_if_loaded(&entry.position)
                .is_none()
            {
                continue;
            }
            entry.block_entity.tick(world).await;
        }
    }
}

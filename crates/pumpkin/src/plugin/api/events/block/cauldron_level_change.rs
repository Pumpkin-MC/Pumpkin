use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::{entity::EntityBase, world::World};

/// Reason why a cauldron's fluid level changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CauldronChangeReason {
    BucketEmpty,
    BucketFill,
    BottleEmpty,
    BottleFill,
    NaturalFill,
    Extinguish,
    Unknown,
}

/// An event that occurs when a cauldron's level changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct CauldronLevelChangeEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub old_level: i32,
    pub new_level: i32,
    pub reason: CauldronChangeReason,
    pub entity: Option<Arc<dyn EntityBase>>,
}

impl CauldronLevelChangeEvent {
    #[must_use]
    pub const fn new(
        block_pos: BlockPos,
        world: Arc<World>,
        old_level: i32,
        new_level: i32,
        reason: CauldronChangeReason,
        entity: Option<Arc<dyn EntityBase>>,
    ) -> Self {
        Self {
            block_pos,
            world,
            old_level,
            new_level,
            reason,
            entity,
            cancelled: false,
        }
    }
}

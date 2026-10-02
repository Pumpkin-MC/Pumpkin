use std::sync::Arc;

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::EntityBase;

use super::position_tracker::{BlockPosTracker, EntityTracker, PositionTracker};

#[derive(Debug)]
pub struct WalkTarget {
    target: Arc<dyn PositionTracker>,
    speed_modifier: f32,
    close_enough_dist: i32,
}

impl WalkTarget {
    #[must_use]
    pub const fn new(
        target: Arc<dyn PositionTracker>,
        speed_modifier: f32,
        close_enough_dist: i32,
    ) -> Self {
        Self {
            target,
            speed_modifier,
            close_enough_dist,
        }
    }

    #[must_use]
    pub fn from_block_pos(target: BlockPos, speed_modifier: f32, close_enough_dist: i32) -> Self {
        Self::new(
            Arc::new(BlockPosTracker::new(target)),
            speed_modifier,
            close_enough_dist,
        )
    }

    #[must_use]
    pub fn from_vec(target: Vector3<f64>, speed_modifier: f32, close_enough_dist: i32) -> Self {
        Self::new(
            Arc::new(BlockPosTracker::new(BlockPos::floored_v(target))),
            speed_modifier,
            close_enough_dist,
        )
    }

    #[must_use]
    pub fn from_entity(
        target: Arc<dyn EntityBase>,
        speed_modifier: f32,
        close_enough_dist: i32,
    ) -> Self {
        Self::new(
            Arc::new(EntityTracker::new(target, false)),
            speed_modifier,
            close_enough_dist,
        )
    }

    #[must_use]
    pub fn target(&self) -> &Arc<dyn PositionTracker> {
        &self.target
    }

    #[must_use]
    pub const fn speed_modifier(&self) -> f32 {
        self.speed_modifier
    }

    #[must_use]
    pub const fn close_enough_dist(&self) -> i32 {
        self.close_enough_dist
    }
}

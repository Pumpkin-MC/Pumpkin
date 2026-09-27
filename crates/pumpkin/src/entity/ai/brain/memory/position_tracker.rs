use std::fmt;
use std::sync::Arc;

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::EntityBase;
use crate::entity::ai::brain::VisibilityContext;

use super::types;
use super::value::describe_entity;

pub trait PositionTracker: Send + Sync + fmt::Debug {
    fn current_position(&self) -> Vector3<f64>;
    fn current_block_position(&self) -> BlockPos;
    fn is_visible_by(&self, ctx: &VisibilityContext<'_>) -> bool;
}

pub struct EntityTracker {
    entity: Arc<dyn EntityBase>,
    track_eye_height: bool,
    target_eye_height: bool,
}

impl EntityTracker {
    #[must_use]
    pub const fn new(entity: Arc<dyn EntityBase>, track_eye_height: bool) -> Self {
        Self {
            entity,
            track_eye_height,
            target_eye_height: false,
        }
    }

    #[must_use]
    pub const fn with_target_eye_height(
        entity: Arc<dyn EntityBase>,
        track_eye_height: bool,
        target_eye_height: bool,
    ) -> Self {
        Self {
            entity,
            track_eye_height,
            target_eye_height,
        }
    }

    #[must_use]
    pub fn entity(&self) -> &Arc<dyn EntityBase> {
        &self.entity
    }
}

impl PositionTracker for EntityTracker {
    fn current_position(&self) -> Vector3<f64> {
        if self.track_eye_height {
            self.entity.get_eye_pos()
        } else {
            self.entity.get_entity().pos.load()
        }
    }

    fn current_block_position(&self) -> BlockPos {
        if self.target_eye_height {
            BlockPos::floored_v(self.entity.get_eye_pos())
        } else {
            self.entity.get_entity().block_pos.load()
        }
    }

    fn is_visible_by(&self, ctx: &VisibilityContext<'_>) -> bool {
        if self.entity.get_living_entity().is_none() {
            return true;
        }
        if !self.entity.get_entity().is_alive() {
            return false;
        }
        ctx.brain
            .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
            .is_some_and(|visible| visible.contains(self.entity.as_ref(), ctx))
    }
}

impl fmt::Debug for EntityTracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EntityTracker for {}",
            describe_entity(self.entity.as_ref())
        )
    }
}

#[derive(Debug)]
pub struct BlockPosTracker {
    block_pos: BlockPos,
    center: Vector3<f64>,
}

impl BlockPosTracker {
    #[must_use]
    pub fn new(block_pos: BlockPos) -> Self {
        Self {
            block_pos,
            center: block_pos.to_centered_f64(),
        }
    }

    #[must_use]
    pub const fn from_vec(center: Vector3<f64>) -> Self {
        Self {
            block_pos: BlockPos::floored_v(center),
            center,
        }
    }
}

impl PositionTracker for BlockPosTracker {
    fn current_position(&self) -> Vector3<f64> {
        self.center
    }

    fn current_block_position(&self) -> BlockPos {
        self.block_pos
    }

    fn is_visible_by(&self, _ctx: &VisibilityContext<'_>) -> bool {
        true
    }
}

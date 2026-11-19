use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use rand::seq::SliceRandom;

use super::super::OneShot;
use crate::entity::ai::brain::memory::{MemoryStatus, types, walk_target::WalkTarget};

#[must_use]
pub fn create(speed_modifier: f32) -> OneShot {
    OneShot::new(
        "InsideBrownianWalk",
        vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
        move |tick| {
            let body_pos = tick.mob.get_entity().block_pos.load();
            if tick.world.can_see_sky(&body_pos) {
                return false;
            }
            let mut poses: Vec<BlockPos> =
                BlockPos::between_closed(body_pos.add(-1, -1, -1), body_pos.add(1, 1, 1)).collect();
            poses.shuffle(&mut tick.mob.get_random());
            let target = poses
                .into_iter()
                .filter(|pos| !tick.world.can_see_sky(pos))
                .filter(|pos| {
                    tick.world
                        .get_block_state(&pos.down())
                        .is_side_solid(BlockDirection::Up)
                })
                .find(|_| {
                    tick.world
                        .is_space_empty(tick.mob.get_entity().bounding_box.load())
                });
            if let Some(target) = target {
                tick.state.set(
                    types::WALK_TARGET,
                    WalkTarget::from_block_pos(target, speed_modifier, 0),
                );
            }
            true
        },
    )
}

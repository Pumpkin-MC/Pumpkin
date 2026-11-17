use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::ChunkHeightmapType;
use rand::RngExt;

use super::super::super::BrainTick;
use super::super::super::memory::{MemoryModuleId, MemoryStatus, types, walk_target::WalkTarget};
use super::super::OneShot;
use crate::world::World;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 1] =
    [(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)];

fn no_blocks_above(can_see_sky: bool, heightmap_y: i32, body_y: f64) -> bool {
    can_see_sky && f64::from(heightmap_y) <= body_y
}

fn get_outdoor_position(tick: &BrainTick<'_>) -> Option<Vector3<f64>> {
    let mut random = tick.mob.get_random();
    let pos = tick.mob.get_entity().block_pos.load();
    let body_y = tick.mob.get_entity().pos.load().y;

    for _ in 0..10 {
        let random_pos = pos.offset(Vector3::new(
            random.random_range(0..20) - 10,
            random.random_range(0..6) - 3,
            random.random_range(0..20) - 10,
        ));
        if has_no_blocks_above(tick.world, body_y, &random_pos) {
            return Some(random_pos.to_f64());
        }
    }

    None
}

#[must_use]
pub fn has_no_blocks_above(world: &World, body_y: f64, target: &BlockPos) -> bool {
    no_blocks_above(
        world.can_see_sky(target),
        world.get_heightmap_height(ChunkHeightmapType::MotionBlocking, target.0.x, target.0.z) + 1,
        body_y,
    )
}

#[must_use]
pub fn create(speed_modifier: f32) -> OneShot {
    OneShot::new(
        "MoveToSkySeeingSpot",
        ENTRY_CONDITIONS.to_vec(),
        move |tick| {
            if tick
                .world
                .can_see_sky(&tick.mob.get_entity().block_pos.load())
            {
                return false;
            }
            if let Some(pos) = get_outdoor_position(tick) {
                tick.state.set(
                    types::WALK_TARGET,
                    WalkTarget::from_vec(pos, speed_modifier, 0),
                );
            }
            true
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_blocks_above_needs_sky_and_a_heightmap_at_or_below_the_body() {
        assert!(!no_blocks_above(false, 63, 64.0));
        assert!(no_blocks_above(true, 64, 64.0));
        assert!(no_blocks_above(true, 63, 64.0));
        assert!(!no_blocks_above(true, 65, 64.0));
    }
}

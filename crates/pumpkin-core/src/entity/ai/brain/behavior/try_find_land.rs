use std::sync::Arc;

use pumpkin_data::tag;
use pumpkin_data::{BlockDirection, HorizontalFacingExt};
use pumpkin_util::math::position::{BlockPos, OutwardIterator};

use super::super::BrainTick;
use super::super::memory::{
    MemoryModuleId, MemoryStatus,
    position_tracker::{BlockPosTracker, PositionTracker},
    types,
    walk_target::WalkTarget,
};
use super::OneShot;

const COOLDOWN_TICKS: i64 = 60;
const NEAR_LIQUID_COOLDOWN_TICKS: i64 = 40;

fn conditions() -> Vec<(MemoryModuleId, MemoryStatus)> {
    vec![
        (types::ATTACK_TARGET.id(), MemoryStatus::ValueAbsent),
        (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
        (types::LOOK_TARGET.id(), MemoryStatus::Registered),
    ]
}

/// Positions around `center` in Manhattan order, skipping its own column.
fn candidates(center: BlockPos, range: i32) -> impl Iterator<Item = BlockPos> {
    OutwardIterator::new(center, range, range, range)
        .filter(move |pos| pos.0.x != center.0.x || pos.0.z != center.0.z)
}

fn has_no_collision(tick: &BrainTick<'_>, pos: &BlockPos) -> bool {
    tick.world
        .get_block_state(pos)
        .get_block_collision_shapes()
        .next()
        .is_none()
}

fn set_targets(tick: &mut BrainTick<'_>, pos: BlockPos, speed_modifier: f32, close_enough: i32) {
    let tracker: Arc<dyn PositionTracker> = Arc::new(BlockPosTracker::new(pos));
    tick.state.set(types::LOOK_TARGET, tracker.clone());
    tick.state.set(
        types::WALK_TARGET,
        WalkTarget::new(tracker, speed_modifier, close_enough),
    );
}

/// Walks a mob standing in water to a dry spot it can stand on.
#[must_use]
pub fn create(range: i32, speed_modifier: f32) -> OneShot {
    let mut next_ok_start_time = 0;
    OneShot::new("TryFindLand", conditions(), move |tick| {
        let body_pos = tick.mob.get_entity().block_pos.load();
        if !tick
            .world
            .fluid_state_has_tag(&body_pos, &tag::Fluid::MINECRAFT_WATER)
        {
            return false;
        }
        if tick.time < next_ok_start_time {
            next_ok_start_time = tick.time + COOLDOWN_TICKS;
            return true;
        }
        let found = candidates(body_pos, range).find(|pos| {
            tick.world.get_fluid(pos).id == pumpkin_data::fluid::Fluid::EMPTY.id
                && has_no_collision(tick, pos)
                && tick
                    .world
                    .get_block_state(&pos.down())
                    .is_side_solid(BlockDirection::Up)
        });
        if let Some(pos) = found {
            set_targets(tick, pos, speed_modifier, 1);
        }
        next_ok_start_time = tick.time + COOLDOWN_TICKS;
        true
    })
}

/// Walks a mob on land to a spot next to an open `fluid_tag` surface.
#[must_use]
pub fn create_near_liquid(
    range: i32,
    speed_modifier: f32,
    fluid_tag: &'static tag::Tag,
) -> OneShot {
    let mut next_ok_start_time = 0;
    OneShot::new("TryFindLandNearLiquid", conditions(), move |tick| {
        let body_pos = tick.mob.get_entity().block_pos.load();
        if tick.world.fluid_state_has_tag(&body_pos, fluid_tag) {
            return false;
        }
        if tick.time < next_ok_start_time {
            next_ok_start_time = tick.time + NEAR_LIQUID_COOLDOWN_TICKS;
            return true;
        }
        let found = candidates(body_pos, range).find(|pos| {
            has_no_collision(tick, pos)
                && !has_no_collision(tick, &pos.down())
                && BlockDirection::horizontal_worldgen().iter().any(|facing| {
                    let side = pos.offset(facing.to_block_direction().to_offset());
                    tick.world.get_block_state(&side).is_air()
                        && tick.world.fluid_state_has_tag(&side.down(), fluid_tag)
                })
        });
        if let Some(pos) = found {
            set_targets(tick, pos, speed_modifier, 0);
        }
        next_ok_start_time = tick.time + NEAR_LIQUID_COOLDOWN_TICKS;
        true
    })
}

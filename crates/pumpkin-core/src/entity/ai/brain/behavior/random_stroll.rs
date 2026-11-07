use pumpkin_data::fluid::Fluid;
use pumpkin_util::math::vector3::Vector3;

use super::super::memory::{MemoryStatus, types, walk_target::WalkTarget};
use super::OneShot;
use super::utils::get_random_swimmable_pos;
use crate::entity::ai::util::{air_and_water_random_pos, goal_utils, land_random_pos};
use crate::entity::mob::Mob;

const MAX_XZ_DIST: i32 = 10;
const MAX_Y_DIST: i32 = 7;
const SWIM_XY_DISTANCE_TIERS: [(i32, i32); 6] = [(1, 1), (3, 3), (5, 5), (6, 5), (7, 7), (10, 7)];

#[must_use]
pub fn stroll(speed_modifier: f32) -> OneShot {
    stroll_with_water(speed_modifier, true)
}

/// [`stroll`] that, unless `may_stroll_from_water`, only starts out of water.
#[must_use]
pub fn stroll_with_water(speed_modifier: f32, may_stroll_from_water: bool) -> OneShot {
    OneShot::new(
        "RandomStroll",
        vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
        move |tick| {
            if !may_stroll_from_water && tick.mob.get_entity().is_in_water() {
                return false;
            }
            let target = land_random_pos::get_pos(tick.mob, MAX_XZ_DIST, MAX_Y_DIST);
            tick.state.set_optional(
                types::WALK_TARGET,
                target.map(|pos| WalkTarget::from_vec(pos, speed_modifier, 0)),
            );
            true
        },
    )
}

#[must_use]
pub fn swim(speed_modifier: f32) -> OneShot {
    OneShot::new(
        "RandomStroll",
        vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
        move |tick| {
            if !tick.mob.get_entity().is_in_water() {
                return false;
            }
            let target = get_target_swim_pos(tick.mob);
            tick.state.set_optional(
                types::WALK_TARGET,
                target.map(|pos| WalkTarget::from_vec(pos, speed_modifier, 0)),
            );
            true
        },
    )
}

#[must_use]
pub fn fly(speed_modifier: f32) -> OneShot {
    OneShot::new(
        "RandomStroll",
        vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
        move |tick| {
            let target = get_target_fly_pos(tick.mob, MAX_XZ_DIST, MAX_Y_DIST);
            tick.state.set_optional(
                types::WALK_TARGET,
                target.map(|pos| WalkTarget::from_vec(pos, speed_modifier, 0)),
            );
            true
        },
    )
}

fn get_target_swim_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
    let entity = mob.get_entity();
    let world = entity.world.load();
    let mut fallback: Option<Vector3<f64>> = None;
    let mut target = None;
    for (horizontal, vertical) in SWIM_XY_DISTANCE_TIERS {
        let position = entity.pos.load();
        target = fallback.map_or_else(
            || get_random_swimmable_pos(mob, horizontal, vertical),
            |fallback| {
                Some(
                    position
                        + (fallback - position).normalize().multiply(
                            f64::from(horizontal),
                            f64::from(vertical),
                            f64::from(horizontal),
                        ),
                )
            },
        );
        let restrict = goal_utils::mob_restricted(mob, f64::from(horizontal));
        let Some(pos) = target else {
            return fallback;
        };
        let block_pos = pos.to_block_pos();
        if world.get_fluid(&block_pos) == &Fluid::EMPTY
            || goal_utils::is_restricted(restrict, mob, &block_pos)
        {
            return fallback;
        }
        fallback = target;
    }
    target
}

fn get_target_fly_pos(
    mob: &dyn Mob,
    max_horizontal_distance: i32,
    max_vertical_distance: i32,
) -> Option<Vector3<f64>> {
    let entity = mob.get_entity();
    let wander_direction =
        Vector3::rotation_vector(f64::from(entity.pitch.load()), f64::from(entity.yaw.load()));
    air_and_water_random_pos::get_pos(
        mob,
        max_horizontal_distance,
        max_vertical_distance,
        -2,
        wander_direction.x,
        wander_direction.z,
        f64::from(std::f32::consts::FRAC_PI_2),
    )
}

// Ported from vanilla Minecraft's FleeSunGoal (net.minecraft.world.entity.ai.goal.FleeSunGoal)
// See: /tmp/pumpkin-vanilla-26.2/decompiled/net/minecraft/world/entity/ai/goal/FleeSunGoal.java
//
// The current Pumpkin registration is for skeletons, which inherit Monster's
// getWalkTargetValue implementation. Keep that calculation local until the
// generic PathfinderMob value is ported for every mob type.

use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

const SEARCH_RANGE: i32 = 10;
const SEARCH_HEIGHT: i32 = 3;
const SEARCH_ATTEMPTS: usize = 10;
const BRIGHT_OUTSIDE_THRESHOLD: u8 = 4;

/// Vanilla `Monster#getWalkTargetValue` delegates to
/// `LevelReader#getPathfindingCostFromLightLevels` and negates the result.
fn monster_walk_target_value(
    world: &crate::world::World,
    pos: &pumpkin_util::math::position::BlockPos,
) -> f32 {
    let brightness = f32::from(world.get_max_local_raw_brightness(pos)) / 15.0;
    let curved_brightness = brightness / (4.0 - 3.0 * brightness);
    let light_value = curved_brightness + world.dimension.ambient_light * (1.0 - curved_brightness);
    -(light_value - 0.5)
}

pub struct FleeSunGoal {
    speed: f64,
    goal_control: Controls,
    target: Option<Vector3<f64>>,
}

impl FleeSunGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            speed,
            goal_control: Controls::MOVE,
            target: None,
        })
    }

    fn find_shelter(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let world = entity.world.load();
        let current_pos = entity.block_pos.load();
        let mut rng = mob.get_random();

        for _ in 0..SEARCH_ATTEMPTS {
            // Vanilla: random.nextInt(20) - 10 → range [-10, 9], random.nextInt(6) - 3 → range [-3, 2]
            // Rust half-open ranges: -10..10 and -3..3
            let dx = rng.random_range(-SEARCH_RANGE..SEARCH_RANGE);
            let dy = rng.random_range(-SEARCH_HEIGHT..SEARCH_HEIGHT);
            let dz = rng.random_range(-SEARCH_RANGE..SEARCH_RANGE);

            let candidate_pos = current_pos.add(dx, dy, dz);

            // Position must not see the sky (sheltered from sun)
            if world.can_see_sky(&candidate_pos) {
                continue;
            }

            // Vanilla's skeleton is a Monster, whose getWalkTargetValue is
            // negative when the pathfinding light cost is positive.
            if monster_walk_target_value(&world, &candidate_pos) >= 0.0 {
                continue;
            }

            return Some(Vector3::new(
                f64::from(candidate_pos.0.x) + 0.5,
                f64::from(candidate_pos.0.y),
                f64::from(candidate_pos.0.z) + 0.5,
            ));
        }

        None
    }
}

impl Goal for FleeSunGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let mob_entity = mob.get_mob_entity();
            let living = &mob_entity.living_entity;
            let entity = &living.entity;

            // Must have no target to avoid sun-burning while fighting
            let target_lock = mob_entity.target.lock().await;
            if target_lock.is_some() {
                return false;
            }
            drop(target_lock);

            // Vanilla Entity.isOnFire() uses remaining fire ticks on the
            // server; the shared visual-fire flag is only a client-side aid.
            if entity.entity_type.fire_immune
                || entity.fire_immune.load(Relaxed)
                || entity.fire_ticks.load(Relaxed) <= 0
            {
                return false;
            }

            let world = entity.world.load();

            // Must be bright outside (daylight)
            let is_fixed_time = world.dimension.fixed_time.is_some();
            let sky_darken = world.sky_darken.load(Relaxed);
            if is_fixed_time || sky_darken >= BRIGHT_OUTSIDE_THRESHOLD {
                return false;
            }

            // Must be able to see the sky from current position (exposed to sun)
            let current_pos = entity.block_pos.load();
            if !world.can_see_sky(&current_pos) {
                return false;
            }

            // Cannot have head armor (which protects from sun)
            if let Ok(eq) = living.entity_equipment.try_lock() {
                use pumpkin_data::data_component_impl::EquipmentSlot;
                let head_item = eq.get(&EquipmentSlot::HEAD);
                if let Ok(stack) = head_item.try_lock()
                    && !stack.is_empty()
                {
                    return false;
                }
            }

            // Try to find a shelter to flee to
            self.target = Self::find_shelter(mob);
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let navigator = mob.get_mob_entity().navigator.lock().unwrap();
            !navigator.is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target {
                let entity = &mob.get_mob_entity().living_entity.entity;
                let pos = entity.pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(pos, target, self.speed));
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

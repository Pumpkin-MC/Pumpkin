// Ported from vanilla Minecraft's FleeSunGoal (net.minecraft.world.entity.ai.goal.FleeSunGoal)
// See: /tmp/pumpkin-vanilla-26.2/decompiled/net/minecraft/world/entity/ai/goal/FleeSunGoal.java
//
// PARITY NOTE: Vanilla's getHidePos uses getWalkTargetValue() to filter for pathfinder-friendly dark spots.
// Pumpkin lacks the getWalkTargetValue() primitive (returns -getPathfindingCostFromLightLevels), so we
// approximate with sky light level checks. This may select different shelter spots than vanilla.
// TODO: Port getWalkTargetValue() from vanilla's PathfinderMob/Mob for full parity.

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

            // Position must have relatively low light level (dark shelter)
            // We use sky light level < BRIGHT_OUTSIDE_THRESHOLD as a proxy for darkness
            if world.get_sky_light_level(&candidate_pos) >= BRIGHT_OUTSIDE_THRESHOLD {
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

            // Must be on fire (visual fire flag) to activate this goal
            // Note: fire_ticks tracks damage-tick counter; has_visual_fire tracks the on-fire visual state
            if !entity.has_visual_fire.load(Relaxed) {
                return false;
            }

            let world = entity.world.load();

            // Must be bright outside (daylight)
            let sky_darken = world.sky_darken.load(Relaxed);
            if sky_darken >= BRIGHT_OUTSIDE_THRESHOLD {
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

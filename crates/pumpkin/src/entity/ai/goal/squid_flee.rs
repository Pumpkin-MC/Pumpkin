use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::particle::Particle;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;

/// Vanilla: `Squid.SquidFleeGoal` (`Squid.java:239-292`). Squids flee from whatever last
/// damaged them, driving movement directly through a `movementVector` applied in `aiStep`
/// with bubble particles every 10 ticks.
///
/// Pumpkin's `Mob::custom_travel` hook applies the published vector exactly like vanilla's
/// `Squid.travel`, so this goal only has to choose that vector.
const FLEE_SPEED: f64 = 3.0;
const FLEE_RANGE_SQ: f64 = 100.0;
const FLEE_MIN_DISTANCE: f64 = 5.0;

pub struct SquidFleeGoal {
    flee_ticks: i32,
}

impl SquidFleeGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self { flee_ticks: 0 })
    }

    fn attacker_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let living = &mob.get_mob_entity().living_entity;
        let attacker_id = living.last_attacker_id.load(Relaxed);
        if attacker_id == 0 {
            return None;
        }
        let world = living.entity.world.load();
        let attacker = world.get_entity_by_id(attacker_id)?;
        Some(attacker.get_entity().pos.load())
    }

    fn can_flee(mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        entity.touching_water.load(Relaxed)
            && Self::attacker_pos(mob).is_some_and(|attacker_pos| {
                entity.pos.load().squared_distance_to_vec(&attacker_pos) < FLEE_RANGE_SQ
            })
    }
}

impl Goal for SquidFleeGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { Self::can_flee(mob) })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { Self::can_flee(mob) })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.flee_ticks = 0;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.flee_ticks += 1;

            let Some(attacker_pos) = Self::attacker_pos(mob) else {
                return;
            };

            let entity = mob.get_entity();
            let my_pos = entity.pos.load();
            let mut flee_to = my_pos - attacker_pos;
            let target = my_pos + flee_to;

            let world = entity.world.load();
            let target_block = BlockPos::new(
                target.x.floor() as i32,
                target.y.floor() as i32,
                target.z.floor() as i32,
            );
            let state = world.get_block_state(&target_block);
            if state.is_liquid() || state.is_air() {
                let length = flee_to.length();
                if length > 0.0 {
                    // Vanilla scales the normalized flee vector after five blocks, then clears
                    // its vertical component when the candidate block is air.
                    let mut speed = FLEE_SPEED;
                    if length > FLEE_MIN_DISTANCE {
                        speed = (speed - (length - FLEE_MIN_DISTANCE) / FLEE_MIN_DISTANCE).max(0.1);
                    }
                    if state.is_air() {
                        flee_to.y = 0.0;
                    }
                    let movement = flee_to.normalize() * (speed / 20.0);
                    mob.set_movement_vector(movement);
                }
            }

            if self.flee_ticks % 10 == 5 {
                world.spawn_particle(
                    my_pos,
                    Vector3::new(0.0, 0.0, 0.0),
                    0.0,
                    1,
                    Particle::Bubble,
                );
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        // Vanilla's squid goals share a movement vector rather than claiming Goal flags.
        Controls::empty()
    }
}

#[cfg(test)]
mod test {
    use super::FLEE_MIN_DISTANCE;

    #[test]
    fn flee_min_distance_matches_vanilla() {
        assert_eq!(FLEE_MIN_DISTANCE, 5.0);
    }
}

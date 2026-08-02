use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;

const CHASE_DISTANCE_SQ: f64 = 16.0; // 4 * 4
const LEASH_DISTANCE_SQ: f64 = 225.0; // 15 * 15
const ATTACK_COOLDOWN_TICKS: i32 = 20;

/// Calculate the speed modifier based on distance to target.
/// Vanilla source: `OcelotAttackGoal.tick()`
const fn get_speed_modifier(melee_range_sq: f64, distance_sq: f64) -> f64 {
    if distance_sq > melee_range_sq && distance_sq < CHASE_DISTANCE_SQ {
        1.33 // Chase at high speed when at medium distance
    } else if distance_sq < LEASH_DISTANCE_SQ {
        0.6 // Slow walk normally
    } else {
        0.8 // Default speed
    }
}

pub struct OcelotAttackGoal {
    goal_control: Controls,
    attack_time: i32,
}

impl OcelotAttackGoal {
    #[must_use]
    pub fn new() -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            attack_time: 0,
        }
    }
}

impl Default for OcelotAttackGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for OcelotAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await;
            target.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await;

            let Some(target) = target.as_ref() else {
                return false;
            };

            if !target.get_entity().is_alive() {
                return false;
            }

            let distance_sq = mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load());

            // Check if target is too far away
            if distance_sq > LEASH_DISTANCE_SQ {
                return false;
            }

            // Continue if navigation isn't done (still moving towards target)
            let nav_idle = mob
                .get_mob_entity()
                .navigator
                .try_lock()
                .is_ok_and(|nav| nav.is_idle());

            !nav_idle
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await.clone();

            let Some(target) = target else {
                return;
            };

            // Update look direction
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);

            // Calculate distance
            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let distance_sq = mob_pos.squared_distance_to_vec(&target_pos);

            // Calculate melee range based on ocelot's bounding box
            let entity_dimension = mob.get_entity().entity_dimension.load();
            let width = entity_dimension.width as f64;
            let melee_range_sq = (width * 2.0) * (width * 2.0);

            // Determine speed modifier
            let speed_modifier = get_speed_modifier(melee_range_sq, distance_sq);

            // Move towards target
            mob.get_mob_entity().navigator.lock().unwrap().set_progress(
                crate::entity::ai::pathfinder::NavigatorGoal {
                    current_progress: mob_pos,
                    destination: target_pos,
                    speed: speed_modifier,
                },
            );

            // Decrement attack cooldown
            self.attack_time = (self.attack_time - 1).max(0);

            // Attack if in range and cooldown is done
            if distance_sq <= melee_range_sq && self.attack_time <= 0 {
                self.attack_time = ATTACK_COOLDOWN_TICKS;
                mob.try_attack(target.as_ref()).await;
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

#[cfg(test)]
mod tests {
    use super::get_speed_modifier;

    #[test]
    fn speed_modifiers() {
        // Vanilla OcelotAttackGoal#tick: melee_range_sq is `bbWidth * 2` squared, which for an
        // ocelot is well under CHASE_DISTANCE_SQ (16.0) -- use a realistic small value so all
        // three tiers are actually reachable.
        let melee_range_sq = 2.0;

        // Within melee range - should use 0.6 (normal approach), not 1.33
        assert_eq!(get_speed_modifier(melee_range_sq, 1.0), 0.6);

        // Between melee range and chase distance - should use 1.33 (sprint)
        assert_eq!(get_speed_modifier(melee_range_sq, 10.0), 1.33);

        // Beyond chase distance but within leash distance - should use 0.6
        assert_eq!(get_speed_modifier(melee_range_sq, 20.0), 0.6);

        // Beyond leash distance - should use 0.8 (default)
        assert_eq!(get_speed_modifier(melee_range_sq, 226.0), 0.8);
    }
}

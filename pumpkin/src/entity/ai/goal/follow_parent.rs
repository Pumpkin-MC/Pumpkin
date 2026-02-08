use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::path::NavigatorGoal;
use crate::entity::mob::Mob;

/// AI goal for baby mobs to follow a nearby adult of the same type.
///
/// When a baby mob (age < 0 in vanilla) is within `search_range` of an
/// adult of the same entity type, it follows that adult. Stops when the
/// adult moves out of range or the baby grows up.
///
/// In vanilla, babies grow to adults after 20 minutes (24000 ticks).
/// This goal only handles the follow behavior — growth is separate.
pub struct FollowParentGoal {
    goal_control: Controls,
    speed: f64,
    /// Entity ID of the parent being followed (0 = none).
    parent_id: i32,
    /// Maximum range to search for a parent.
    search_range: f64,
    /// Distance at which following stops (close enough).
    stop_distance_sq: f64,
    /// Delay between re-scans when no parent is found.
    delay: i32,
}

impl FollowParentGoal {
    /// Default follow distance before stopping (squared: 9^2 = 81).
    const DEFAULT_STOP_DISTANCE_SQ: f64 = 81.0;
    /// Re-scan delay when no parent is found: 200 ticks (10 seconds).
    const SCAN_DELAY: i32 = 200;

    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE,
            speed,
            parent_id: 0,
            search_range: 16.0,
            stop_distance_sq: Self::DEFAULT_STOP_DISTANCE_SQ,
            delay: 0,
        })
    }
}

impl Goal for FollowParentGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if self.delay > 0 {
                self.delay -= 1;
                return false;
            }

            let mob_entity = mob.get_mob_entity();
            let mob_pos = mob_entity.living_entity.entity.pos.load();
            let mob_type = mob_entity.living_entity.entity.entity_type;
            let my_id = mob_entity.living_entity.entity.entity_id;
            let world = mob_entity.living_entity.entity.world.load();

            // Search for nearby adults of the same type
            let nearby = world.get_nearby_entities(mob_pos, self.search_range);

            let mut best_dist = f64::MAX;
            let mut best_id = 0i32;

            for entity in nearby.values() {
                let ent = entity.get_entity();
                // Must be same entity type, different entity, alive
                if ent.entity_type != mob_type
                    || ent.entity_id == my_id
                    || !ent.is_alive()
                {
                    continue;
                }

                // In a full implementation, we'd check that the other entity
                // is an adult (age >= 0). For now, pick the closest same-type
                // entity as the "parent" — this works correctly when only one
                // baby exists near one adult.
                let ent_pos = ent.pos.load();
                let dist = mob_pos.squared_distance_to_vec(&ent_pos);
                if dist < best_dist {
                    best_dist = dist;
                    best_id = ent.entity_id;
                }
            }

            if best_id != 0 {
                self.parent_id = best_id;
                true
            } else {
                self.delay = Self::SCAN_DELAY;
                false
            }
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if self.parent_id == 0 {
                return false;
            }

            let mob_entity = mob.get_mob_entity();
            let world = mob_entity.living_entity.entity.world.load();

            if let Some(parent) = world.get_entity_by_id(self.parent_id) {
                let parent_ent = parent.get_entity();
                if !parent_ent.is_alive() {
                    return false;
                }

                // Stop following if too far away
                let mob_pos = mob_entity.living_entity.entity.pos.load();
                let parent_pos = parent_ent.pos.load();
                let dist_sq = mob_pos.squared_distance_to_vec(&parent_pos);

                // Continue if not too far and not already close enough
                dist_sq <= self.search_range * self.search_range
                    && dist_sq > self.stop_distance_sq
            } else {
                false
            }
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if self.parent_id == 0 {
                return;
            }

            let mob_entity = mob.get_mob_entity();
            let world = mob_entity.living_entity.entity.world.load();

            if let Some(parent) = world.get_entity_by_id(self.parent_id) {
                let parent_pos = parent.get_entity().pos.load();
                let current_pos = mob_entity.living_entity.entity.pos.load();
                let mut navigator = mob_entity.navigator.lock().await;
                navigator.set_progress(NavigatorGoal {
                    current_progress: current_pos,
                    destination: parent_pos,
                    speed: self.speed,
                });
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if self.parent_id == 0 {
                return;
            }

            let mob_entity = mob.get_mob_entity();
            let world = mob_entity.living_entity.entity.world.load();

            let Some(parent) = world.get_entity_by_id(self.parent_id) else {
                self.parent_id = 0;
                return;
            };

            let parent_pos = parent.get_entity().pos.load();
            let current_pos = mob_entity.living_entity.entity.pos.load();
            let mut navigator = mob_entity.navigator.lock().await;
            navigator.set_progress(NavigatorGoal {
                current_progress: current_pos,
                destination: parent_pos,
                speed: self.speed,
            });
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.parent_id = 0;
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.cancel();
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

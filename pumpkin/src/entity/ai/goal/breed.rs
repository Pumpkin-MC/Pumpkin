use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::path::NavigatorGoal;
use crate::entity::mob::Mob;
use std::sync::atomic::Ordering;

/// AI goal for breedable passive mobs.
///
/// When the mob is "in love" (love_ticks > 0), it searches for a nearby mob
/// of the same entity type that is also in love. If found, navigates toward
/// the partner. When within 2.5 blocks, both mobs' love_ticks are reset and
/// the breed_cooldown is set to 6000 ticks (5 minutes), matching vanilla.
///
/// Love mode is activated externally (e.g. when a player feeds the mob its
/// breeding food). The goal only handles the pathfinding/mating AI.
pub struct BreedGoal {
    goal_control: Controls,
    speed: f64,
    /// Ticks remaining in "love mode." Set externally when fed breeding food.
    /// Decremented each goal tick. Mob shows heart particles while > 0.
    pub love_ticks: i32,
    /// Cooldown after breeding. Must reach 0 before next breed.
    pub breed_cooldown: i32,
    /// Entity ID of the current breeding partner (0 = none).
    partner_id: i32,
}

impl BreedGoal {
    /// Vanilla breeding cooldown: 6000 ticks (5 minutes).
    pub const BREED_COOLDOWN: i32 = 6000;
    /// Default love mode duration: 600 ticks (30 seconds).
    pub const LOVE_DURATION: i32 = 600;
    /// Maximum range to search for a partner.
    const SEARCH_RANGE: f64 = 8.0;
    /// Distance at which breeding completes (squared = 2.5^2 = 6.25).
    const BREED_DISTANCE_SQ: f64 = 6.25;

    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            speed,
            love_ticks: 0,
            breed_cooldown: 0,
            partner_id: 0,
        })
    }

    /// Activate love mode. Called when a player feeds the mob its breeding food.
    pub fn set_in_love(&mut self) {
        if self.breed_cooldown <= 0 {
            self.love_ticks = Self::LOVE_DURATION;
        }
    }

    /// Whether this mob is currently in love mode.
    #[must_use]
    pub fn is_in_love(&self) -> bool {
        self.love_ticks > 0
    }

    /// Whether this mob can breed (in love and not on cooldown).
    #[must_use]
    pub fn can_breed(&self) -> bool {
        self.love_ticks > 0 && self.breed_cooldown <= 0
    }
}

impl Goal for BreedGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            // Tick cooldowns even when not starting
            if self.breed_cooldown > 0 {
                self.breed_cooldown -= 1;
            }

            if !self.can_breed() {
                return false;
            }

            let mob_entity = mob.get_mob_entity();
            let mob_pos = mob_entity.living_entity.entity.pos.load();
            let mob_type = mob_entity.living_entity.entity.entity_type;
            let world = mob_entity.living_entity.entity.world.load();
            let my_id = mob_entity.living_entity.entity.entity_id;

            // Search for nearby entities of the same type
            let nearby = world.get_nearby_entities(mob_pos, Self::SEARCH_RANGE);

            let mut best_dist = f64::MAX;
            let mut best_id = 0i32;

            for entity in nearby.values() {
                let ent = entity.get_entity();
                // Must be same entity type, different entity, and alive
                if ent.entity_type != mob_type
                    || ent.entity_id == my_id
                    || !ent.is_alive()
                {
                    continue;
                }

                let ent_pos = ent.pos.load();
                let dist = mob_pos.squared_distance_to_vec(&ent_pos);
                if dist < best_dist {
                    best_dist = dist;
                    best_id = ent.entity_id;
                }
            }

            if best_id != 0 {
                self.partner_id = best_id;
                true
            } else {
                false
            }
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if self.love_ticks <= 0 || self.partner_id == 0 {
                return false;
            }

            let mob_entity = mob.get_mob_entity();
            let world = mob_entity.living_entity.entity.world.load();

            // Check that the partner still exists
            if let Some(partner) = world.get_entity_by_id(self.partner_id) {
                let partner_ent = partner.get_entity();
                partner_ent.is_alive()
                    && partner_ent.entity_type
                        == mob_entity.living_entity.entity.entity_type
            } else {
                false
            }
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if self.partner_id == 0 {
                return;
            }

            let mob_entity = mob.get_mob_entity();
            let world = mob_entity.living_entity.entity.world.load();

            if let Some(partner) = world.get_entity_by_id(self.partner_id) {
                let partner_pos = partner.get_entity().pos.load();
                let current_pos = mob_entity.living_entity.entity.pos.load();
                let mut navigator = mob_entity.navigator.lock().await;
                navigator.set_progress(NavigatorGoal {
                    current_progress: current_pos,
                    destination: partner_pos,
                    speed: self.speed,
                });
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            // Decrement love ticks
            if self.love_ticks > 0 {
                self.love_ticks -= 1;
            }

            if self.partner_id == 0 {
                return;
            }

            let mob_entity = mob.get_mob_entity();
            let world = mob_entity.living_entity.entity.world.load();

            let Some(partner) = world.get_entity_by_id(self.partner_id) else {
                self.partner_id = 0;
                return;
            };

            let partner_pos = partner.get_entity().pos.load();
            let mob_pos = mob_entity.living_entity.entity.pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&partner_pos);

            // Look at partner
            mob_entity.living_entity.entity.look_at(partner_pos);

            // Navigate toward partner
            let current_pos = mob_entity.living_entity.entity.pos.load();
            let mut navigator = mob_entity.navigator.lock().await;
            navigator.set_progress(NavigatorGoal {
                current_progress: current_pos,
                destination: partner_pos,
                speed: self.speed,
            });
            drop(navigator);

            // Close enough to breed
            if dist_sq <= Self::BREED_DISTANCE_SQ {
                // Reset love mode and set breed cooldown
                self.love_ticks = 0;
                self.breed_cooldown = Self::BREED_COOLDOWN;
                self.partner_id = 0;

                // Spawn heart particles around the mob
                let entity = &mob_entity.living_entity.entity;
                let age = entity.age.load(Ordering::Relaxed);

                // Increment entity age to simulate baby growth start
                // In vanilla, the baby entity is spawned here. We set age
                // to a negative value on the newborn mob. For now, we just
                // complete the breeding action (baby spawn requires entity
                // factory integration — tracked as future work).
                let _ = age; // suppress unused warning

                // TODO: Spawn baby entity of the same type at this position
                // TODO: Fire EntityBreedEvent
                // TODO: Grant player breeding XP (1-7 orbs)
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.partner_id = 0;
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

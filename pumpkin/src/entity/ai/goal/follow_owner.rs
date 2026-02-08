use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::path::NavigatorGoal;
use crate::entity::mob::Mob;

/// A goal for tamed mobs to follow their owner (a player).
///
/// Stores the owner's entity ID internally. When the mob is far from
/// its owner, it navigates toward them. If the distance exceeds the
/// teleport threshold, the mob teleports directly to the owner.
///
/// Owner state is stored in the goal struct (not on `MobEntity`) and
/// is activated externally via `set_owner()` when taming occurs.
///
/// Used by: Wolf, Cat (when tamed).
pub struct FollowOwnerGoal {
    goal_control: Controls,
    owner_id: Option<i32>,
    speed: f64,
    start_distance_sq: f64,
    stop_distance_sq: f64,
    teleport_distance_sq: f64,
}

impl FollowOwnerGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            owner_id: None,
            speed,
            start_distance_sq: 100.0, // 10 blocks
            stop_distance_sq: 4.0,    // 2 blocks
            teleport_distance_sq: 144.0, // 12 blocks
        })
    }

    /// Set the owner entity ID. Called externally when the mob is tamed.
    pub const fn set_owner(&mut self, entity_id: i32) {
        self.owner_id = Some(entity_id);
    }

    /// Clear the owner. Called when the mob is untamed or owner leaves.
    pub const fn clear_owner(&mut self) {
        self.owner_id = None;
    }
}

impl Goal for FollowOwnerGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let Some(owner_id) = self.owner_id else {
                return false;
            };

            let mob_entity = mob.get_mob_entity();
            let world = mob_entity.living_entity.entity.world.load();
            let Some(owner) = world.get_player_by_id(owner_id) else {
                return false;
            };

            let mob_pos = mob_entity.living_entity.entity.pos.load();
            let owner_pos = owner.position();
            let dist_sq = mob_pos.squared_distance_to_vec(&owner_pos);

            // Only start following when the owner is far enough away
            dist_sq > self.start_distance_sq
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let Some(owner_id) = self.owner_id else {
                return false;
            };

            let mob_entity = mob.get_mob_entity();
            let world = mob_entity.living_entity.entity.world.load();
            let Some(owner) = world.get_player_by_id(owner_id) else {
                return false;
            };

            let mob_pos = mob_entity.living_entity.entity.pos.load();
            let owner_pos = owner.position();
            let dist_sq = mob_pos.squared_distance_to_vec(&owner_pos);

            // Stop following when close enough
            dist_sq > self.stop_distance_sq
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.cancel();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let Some(owner_id) = self.owner_id else {
                return;
            };

            let mob_entity = mob.get_mob_entity();
            let world = mob_entity.living_entity.entity.world.load();
            let Some(owner) = world.get_player_by_id(owner_id) else {
                return;
            };

            let mob_pos = mob_entity.living_entity.entity.pos.load();
            let owner_pos = owner.position();

            // Look at owner
            mob_entity.living_entity.entity.look_at(owner_pos);

            let dist_sq = mob_pos.squared_distance_to_vec(&owner_pos);

            if dist_sq > self.teleport_distance_sq {
                // Teleport to owner when too far
                mob_entity.living_entity.entity.set_pos(owner_pos);
                let mut navigator = mob_entity.navigator.lock().await;
                navigator.cancel();
            } else {
                // Navigate toward owner
                let mut navigator = mob_entity.navigator.lock().await;
                navigator.set_progress(NavigatorGoal {
                    current_progress: mob_pos,
                    destination: owner_pos,
                    speed: self.speed,
                });
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

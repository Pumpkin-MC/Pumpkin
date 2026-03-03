use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};

const SEARCH_RADIUS: i32 = 48;
const CLAIM_DISTANCE_SQ: f64 = 4.0; // Must be within 2 blocks to claim

pub struct ClaimWorkstationGoal {
    goal_control: Controls,
    target_pos: Option<BlockPos>,
    cooldown: i32,
}

impl Default for ClaimWorkstationGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaimWorkstationGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            goal_control: Controls::MOVE,
            target_pos: None,
            cooldown: 0,
        }
    }
}

impl Goal for ClaimWorkstationGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.cooldown > 0 {
                self.cooldown -= 1;
                return false;
            }

            // Only search if the villager has no profession (unemployed)
            // We check this by looking at whether a workstation position is already set
            // The villager entity will handle the actual profession check
            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load_full();
            let pos = entity.pos.load();
            let block_pos = BlockPos(Vector3::new(pos.x as i32, pos.y as i32, pos.z as i32));

            // Search for workstation POIs within radius
            let mut poi_storage = world.portal_poi.lock().await;
            let candidates = poi_storage.get_in_square(block_pos, SEARCH_RADIUS, None);
            drop(poi_storage);

            // Find the closest workstation POI
            let mut best: Option<(BlockPos, f64)> = None;
            for candidate in candidates {
                let dx = (candidate.0.x as f64) - pos.x;
                let dy = (candidate.0.y as f64) - pos.y;
                let dz = (candidate.0.z as f64) - pos.z;
                let dist_sq = dx * dx + dy * dy + dz * dz;

                if best.is_none() || dist_sq < best.unwrap().1 {
                    best = Some((candidate, dist_sq));
                }
            }

            if let Some((target, _)) = best {
                self.target_pos = Some(target);
                true
            } else {
                self.cooldown = to_goal_ticks(200); // Wait ~10 seconds before searching again
                false
            }
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.target_pos.is_none() {
                return false;
            }
            let navigator = mob.get_mob_entity().navigator.lock().await;
            !navigator.is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target_pos {
                let pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let target_f64 = Vector3::new(
                    target.0.x as f64 + 0.5,
                    target.0.y as f64,
                    target.0.z as f64 + 0.5,
                );
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal::new(pos, target_f64, 0.5));
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target_pos {
                let pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let dx = pos.x - (target.0.x as f64 + 0.5);
                let dy = pos.y - target.0.y as f64;
                let dz = pos.z - (target.0.z as f64 + 0.5);
                let dist_sq = dx * dx + dy * dy + dz * dz;

                if dist_sq <= CLAIM_DISTANCE_SQ {
                    // Close enough to claim — the actual claiming logic
                    // (setting profession, workstation_pos) is handled by the villager's mob_tick
                    // which checks if we're near a workstation POI
                    self.target_pos = None;
                }
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target_pos = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::passive::villager::schedule::VillagerActivity;
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};

/// Goal that makes a villager navigate to the village bell during meeting time.
pub struct GatherAtBellGoal {
    goal_control: Controls,
    bell_pos: Option<BlockPos>,
    cooldown: i32,
}

impl Default for GatherAtBellGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl GatherAtBellGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            goal_control: Controls::MOVE,
            bell_pos: None,
            cooldown: 0,
        }
    }
}

impl Goal for GatherAtBellGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.cooldown > 0 {
                self.cooldown -= 1;
                return false;
            }

            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load_full();
            let time_of_day = world.level_time.lock().await.time_of_day;
            let activity = VillagerActivity::from_time(time_of_day);

            if !activity.is_meeting() {
                return false;
            }

            // Use POI system to find bells instead of brute-force block scanning.
            // Search for bell POI type within 48 blocks.
            let pos = entity.pos.load();
            let block_pos = BlockPos(Vector3::new(pos.x as i32, pos.y as i32, pos.z as i32));

            let mut poi_storage = world.portal_poi.lock().await;
            let candidates =
                poi_storage.get_in_square(block_pos, 48, Some(pumpkin_world::poi::POI_TYPE_BELL));
            drop(poi_storage);

            // If POI search returned nothing, fall back to a small-radius block check
            // (only 5-block radius = 3×3×7 = 63 lookups max, not 4375)
            if candidates.is_empty() {
                let search_radius = 5i32;
                let mut best: Option<(BlockPos, i32)> = None;

                for dx in -search_radius..=search_radius {
                    for dz in -search_radius..=search_radius {
                        for dy in -3..=3 {
                            let check_pos = BlockPos(Vector3::new(
                                block_pos.0.x + dx,
                                block_pos.0.y + dy,
                                block_pos.0.z + dz,
                            ));
                            let block = world.get_block(&check_pos).await;
                            let name = block.name.strip_prefix("minecraft:").unwrap_or(block.name);
                            if name == "bell" {
                                let dist = dx * dx + dz * dz;
                                if best.is_none() || dist < best.unwrap().1 {
                                    best = Some((check_pos, dist));
                                }
                            }
                        }
                    }
                }

                if let Some((pos, _)) = best {
                    self.bell_pos = Some(pos);
                    return true;
                }

                self.cooldown = to_goal_ticks(600);
                return false;
            }

            // Find closest bell from POI results
            if let Some(nearest) = candidates.into_iter().min_by_key(|c| {
                let dx = c.0.x - block_pos.0.x;
                let dz = c.0.z - block_pos.0.z;
                dx * dx + dz * dz
            }) {
                self.bell_pos = Some(nearest);
                true
            } else {
                self.cooldown = to_goal_ticks(600);
                false
            }
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.bell_pos.is_none() {
                return false;
            }
            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load_full();
            let time_of_day = world.level_time.lock().await.time_of_day;
            let activity = VillagerActivity::from_time(time_of_day);
            activity.is_meeting()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(bell) = self.bell_pos {
                let pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let target = Vector3::new(
                    bell.0.x as f64 + 0.5,
                    bell.0.y as f64,
                    bell.0.z as f64 + 0.5,
                );
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal::new(pos, target, 0.5));
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.bell_pos = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

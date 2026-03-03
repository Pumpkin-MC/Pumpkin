use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use crate::entity::passive::villager::schedule::VillagerActivity;

/// Goal that makes a villager navigate to the village bell during meeting time.
pub struct GatherAtBellGoal {
    goal_control: Controls,
    bell_pos: Option<BlockPos>,
    cooldown: i32,
}

impl GatherAtBellGoal {
    #[must_use]
    pub fn new() -> Self {
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

            // Search for a bell block nearby
            let pos = entity.pos.load();
            let block_pos = BlockPos(Vector3::new(pos.x as i32, pos.y as i32, pos.z as i32));

            // Search in a 48-block radius for a bell
            // We check blocks directly since bells aren't POI types in our system
            let search_radius = 48i32;
            let mut best: Option<(BlockPos, i32)> = None;

            // Only search in a limited area to avoid performance issues
            // Check a few random positions or use POI if bell POI exists
            // For efficiency, scan in a grid pattern
            for dx in (-search_radius..=search_radius).step_by(4) {
                for dz in (-search_radius..=search_radius).step_by(4) {
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
                true
            } else {
                self.cooldown = to_goal_ticks(600); // Wait longer for bell search
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

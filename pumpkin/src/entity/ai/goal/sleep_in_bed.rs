use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use crate::entity::passive::villager::schedule::VillagerActivity;

/// Goal that makes a villager navigate to and sleep in their bed at night.
pub struct SleepInBedGoal {
    goal_control: Controls,
    bed_pos: Option<BlockPos>,
    cooldown: i32,
}

impl SleepInBedGoal {
    #[must_use]
    pub fn new() -> Self {
        Self {
            goal_control: Controls::MOVE,
            bed_pos: None,
            cooldown: 0,
        }
    }
}

impl Goal for SleepInBedGoal {
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

            if !activity.is_sleeping() {
                return false;
            }

            // Search for a bed POI nearby
            let pos = entity.pos.load();
            let block_pos = BlockPos(Vector3::new(pos.x as i32, pos.y as i32, pos.z as i32));

            let mut poi_storage = world.portal_poi.lock().await;
            let candidates = poi_storage.get_in_square(
                block_pos,
                48,
                Some(pumpkin_world::poi::POI_TYPE_HOME),
            );
            drop(poi_storage);

            if let Some(nearest) = candidates.into_iter().min_by_key(|c| {
                let dx = c.0.x - block_pos.0.x;
                let dz = c.0.z - block_pos.0.z;
                dx * dx + dz * dz
            }) {
                self.bed_pos = Some(nearest);
                true
            } else {
                self.cooldown = to_goal_ticks(200);
                false
            }
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.bed_pos.is_none() {
                return false;
            }
            // Stop sleeping when it's no longer rest time
            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load_full();
            let time_of_day = world.level_time.lock().await.time_of_day;
            let activity = VillagerActivity::from_time(time_of_day);
            activity.is_sleeping()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(bed) = self.bed_pos {
                let pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let target = Vector3::new(
                    bed.0.x as f64 + 0.5,
                    bed.0.y as f64 + 0.5625, // Bed surface height
                    bed.0.z as f64 + 0.5,
                );
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal::new(pos, target, 0.5));
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.bed_pos = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

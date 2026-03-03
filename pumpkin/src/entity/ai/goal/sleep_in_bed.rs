use pumpkin_data::entity::EntityPose;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::passive::villager::schedule::VillagerActivity;
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};

/// Goal that makes a villager navigate to and sleep in their bed at night.
pub struct SleepInBedGoal {
    goal_control: Controls,
    bed_pos: Option<BlockPos>,
    cooldown: i32,
    sleeping: bool,
}

impl Default for SleepInBedGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl SleepInBedGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            goal_control: Controls::MOVE,
            bed_pos: None,
            cooldown: 0,
            sleeping: false,
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
            let block_pos = BlockPos(Vector3::new(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            ));

            let mut poi_storage = world.portal_poi.lock().await;
            let candidates =
                poi_storage.get_in_square(block_pos, 48, Some(pumpkin_world::poi::POI_TYPE_HOME));
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

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if self.sleeping {
                return;
            }
            if let Some(bed) = self.bed_pos {
                let entity = &mob.get_mob_entity().living_entity.entity;
                let pos = entity.pos.load();
                let dx = pos.x - (bed.0.x as f64 + 0.5);
                let dz = pos.z - (bed.0.z as f64 + 0.5);
                let dist_sq = dx * dx + dz * dz;

                if dist_sq <= 2.25 {
                    // Within ~1.5 blocks of bed → sleep
                    self.sleeping = true;
                    // Stop navigation
                    mob.get_mob_entity().navigator.lock().await.stop();
                    // Zero velocity
                    entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
                    // Snap to bed surface
                    entity.set_pos(Vector3::new(
                        bed.0.x as f64 + 0.5,
                        bed.0.y as f64 + 0.5625,
                        bed.0.z as f64 + 0.5,
                    ));
                    // Set sleeping pose
                    entity.set_pose(EntityPose::Sleeping).await;
                }
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if self.sleeping {
                let entity = &mob.get_mob_entity().living_entity.entity;
                entity.set_pose(EntityPose::Standing).await;
                // Reset Y position to standing on top of the bed
                if let Some(bed) = self.bed_pos {
                    let current_pos = entity.pos.load();
                    entity.set_pos(Vector3::new(
                        current_pos.x,
                        bed.0.y as f64 + 1.0,
                        current_pos.z,
                    ));
                }
                self.sleeping = false;
            }
            self.bed_pos = None;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

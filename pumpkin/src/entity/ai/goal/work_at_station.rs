use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::passive::villager::schedule::VillagerActivity;
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};

/// Goal that makes a villager navigate to their workstation during work hours.
pub struct WorkAtStationGoal {
    goal_control: Controls,
    station_pos: Option<BlockPos>,
    cooldown: i32,
    work_ticks: i32,
}

impl Default for WorkAtStationGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkAtStationGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            goal_control: Controls::MOVE,
            station_pos: None,
            cooldown: 0,
            work_ticks: 0,
        }
    }
}

impl Goal for WorkAtStationGoal {
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

            if !activity.is_working() {
                return false;
            }

            // The villager needs to have a workstation position set
            // We check all entities to find ourselves and get the workstation
            // For now, use POI search to find the nearest workstation
            let pos = entity.pos.load();
            let block_pos = BlockPos(Vector3::new(pos.x as i32, pos.y as i32, pos.z as i32));

            // Search for any workstation POI nearby (the villager's own station)
            let mut poi_storage = world.portal_poi.lock().await;
            let candidates = poi_storage.get_in_square(block_pos, 48, None);
            drop(poi_storage);

            // Filter to workstation types only
            let mut best: Option<(BlockPos, i32)> = None;
            for candidate in candidates {
                let block = world.get_block(&candidate).await;
                let short_name = block.name.strip_prefix("minecraft:").unwrap_or(block.name);
                if pumpkin_world::poi::block_to_poi_type(short_name)
                    .and_then(pumpkin_world::poi::poi_type_to_profession)
                    .is_some()
                {
                    let dx = candidate.0.x - block_pos.0.x;
                    let dz = candidate.0.z - block_pos.0.z;
                    let dist = dx * dx + dz * dz;
                    if best.is_none() || dist < best.unwrap().1 {
                        best = Some((candidate, dist));
                    }
                }
            }

            if let Some((pos, _)) = best {
                self.station_pos = Some(pos);
                self.work_ticks = 0;
                true
            } else {
                self.cooldown = to_goal_ticks(200);
                false
            }
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.station_pos.is_none() {
                return false;
            }
            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load_full();
            let time_of_day = world.level_time.lock().await.time_of_day;
            let activity = VillagerActivity::from_time(time_of_day);
            activity.is_working()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(station) = self.station_pos {
                let pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let target = Vector3::new(
                    station.0.x as f64 + 0.5,
                    station.0.y as f64,
                    station.0.z as f64 + 0.5,
                );
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal::new(pos, target, 0.5));
            }
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if self.station_pos.is_some() {
                self.work_ticks += 1;
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.station_pos = None;
            self.work_ticks = 0;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

use std::sync::atomic::AtomicI32;

use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;

pub struct PathfindToRaidGoal {
    recruitment_tick: AtomicI32,
    speed_modifier: f64,
}

impl Default for PathfindToRaidGoal {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl PathfindToRaidGoal {
    #[must_use]
    pub const fn new(speed_modifier: f64) -> Self {
        Self {
            recruitment_tick: AtomicI32::new(0),
            speed_modifier,
        }
    }
}

impl Goal for PathfindToRaidGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(raider) = mob.as_raider() else {
            return false;
        };

        let target = mob.get_mob_entity().get_target().clone();
        if target.is_some() || !raider.has_active_raid() {
            return false;
        }

        let Some(raid_id) = raider.get_raider_data().raid_id.load() else {
            return false;
        };

        let pos = mob.get_entity().block_pos.load();
        let world = mob.get_entity().world.load();
        let is_village = {
            let raids = world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(raid) = raids.get(raid_id) else {
                return false;
            };
            if raid.is_over() {
                return false;
            }
            world
                .villager_poi
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_nearest_job_site(pos, 32)
                .is_some()
        };

        !is_village
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let Some(raider) = mob.as_raider() else {
            return false;
        };

        let target = mob.get_mob_entity().get_target().clone();
        if target.is_some() || !raider.has_active_raid() {
            return false;
        }

        let Some(raid_id) = raider.get_raider_data().raid_id.load() else {
            return false;
        };

        let pos = mob.get_entity().block_pos.load();
        let world = mob.get_entity().world.load();
        let is_village = {
            let raids = world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(raid) = raids.get(raid_id) else {
                return false;
            };
            if raid.is_over() {
                return false;
            }
            world
                .villager_poi
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_nearest_job_site(pos, 32)
                .is_some()
        };

        !is_village
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(raider) = mob.as_raider() else {
            return;
        };

        let Some(raid_id) = raider.get_raider_data().raid_id.load() else {
            return;
        };

        let entity = mob.get_entity();
        let world = entity.world.load();
        let current_age = entity.age.load(std::sync::atomic::Ordering::Relaxed);

        let raid_center = {
            let raids = world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(raid) = raids.get(raid_id) else {
                return;
            };
            if raid.is_over() {
                return;
            }
            raid.center
        };

        // Periodic recruitment of nearby raiders
        let next_recruit = self
            .recruitment_tick
            .load(std::sync::atomic::Ordering::Relaxed);
        if current_age >= next_recruit {
            self.recruitment_tick
                .store(current_age + 20, std::sync::atomic::Ordering::Relaxed);

            let bb = entity.bounding_box.load().expand(16.0, 16.0, 16.0);
            let nearby = world.get_entities_at_box(&bb);

            for cand in nearby {
                if cand.get_entity().entity_id != entity.entity_id
                    && let Some(cand_mob) = cand.get_mob()
                    && let Some(cand_raider) = cand_mob.as_raider()
                    && !cand_raider.has_active_raid()
                    && cand_raider.can_join_raid()
                {
                    cand_raider.get_raider_data().raid_id.store(Some(raid_id));
                }
            }
        }

        // Pathfind towards raid center if idle
        let pos = entity.pos.load();
        let mut nav = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if nav.is_idle() {
            // Generate a point towards the raid center
            let center_vec = Vector3::new(
                f64::from(raid_center.0.x) + 0.5,
                f64::from(raid_center.0.y),
                f64::from(raid_center.0.z) + 0.5,
            );
            let dir = center_vec - pos;
            let dir_len = dir.x.hypot(dir.z);

            let step_dist = 15.0f64.min(dir_len);
            let norm_dir_x = if dir_len > 0.001 {
                dir.x / dir_len
            } else {
                0.0
            };
            let norm_dir_z = if dir_len > 0.001 {
                dir.z / dir_len
            } else {
                0.0
            };

            // Add slight random offset (-45 to +45 degrees)
            let angle = (rand::random::<f64>() - 0.5) * std::f64::consts::FRAC_PI_2;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            let rx = norm_dir_x * cos_a - norm_dir_z * sin_a;
            let rz = norm_dir_x * sin_a + norm_dir_z * cos_a;

            let dest = Vector3::new(pos.x + rx * step_dist, pos.y, pos.z + rz * step_dist);

            nav.set_progress(NavigatorGoal {
                current_progress: pos,
                destination: dest,
                speed: self.speed_modifier,
            });
        }
    }
}

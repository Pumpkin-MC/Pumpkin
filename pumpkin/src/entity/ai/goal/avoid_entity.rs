use std::sync::Arc;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{EntityBase, ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rand::RngExt;

const FAST_DISTANCE_SQ: f64 = 49.0;
const HORIZONTAL_RANGE: f64 = 16.0;
const VERTICAL_RANGE: i32 = 7;

pub struct AvoidEntityGoal {
    goal_control: Controls,
    flee_type: &'static EntityType,
    flee_distance: f64,
    slow_speed: f64,
    fast_speed: f64,
    target: Option<Arc<dyn EntityBase>>,
    flee_pos: Option<Vector3<f64>>,
}

impl AvoidEntityGoal {
    #[must_use]
    pub fn new(
        flee_type: &'static EntityType,
        flee_distance: f64,
        slow_speed: f64,
        fast_speed: f64,
    ) -> Self {
        Self {
            goal_control: Controls::MOVE,
            flee_type,
            flee_distance,
            slow_speed,
            fast_speed,
            target: None,
            flee_pos: None,
        }
    }

    fn find_threat(&self, mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();

        if self.flee_type == &EntityType::PLAYER {
            world
                .get_closest_player(pos, self.flee_distance)
                .map(|p| p as Arc<dyn EntityBase>)
        } else {
            world.get_closest_entity(pos, self.flee_distance, Some(&[self.flee_type]))
        }
    }

    /// Generates a random walkable position within a cone pointing away from the threat.
    /// Mirrors vanilla's `NoPenaltyTargeting.findFrom()`.
    async fn find_flee_position(mob: &dyn Mob, threat_pos: &Vector3<f64>) -> Option<Vector3<f64>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let mob_pos = entity.pos.load();
        let world = entity.world.load();

        let candidates = {
            let mut rng = mob.get_random();
            let dir_x = mob_pos.x - threat_pos.x;
            let dir_z = mob_pos.z - threat_pos.z;
            let (dir_x, dir_z) = if dir_x == 0.0 && dir_z == 0.0 {
                (rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0))
            } else {
                (dir_x, dir_z)
            };
            let base_angle = dir_z.atan2(dir_x) - std::f64::consts::FRAC_PI_2;

            let mut candidates = Vec::with_capacity(10);
            for _ in 0..10 {
                let angle = base_angle
                    + (2.0 * rng.random_range(0.0..1.0) - 1.0) * std::f64::consts::FRAC_PI_2;
                let t = rng.random_range(0.0..1.0f64).sqrt();
                let dist = t * HORIZONTAL_RANGE * std::f64::consts::SQRT_2;
                let dx = -dist * angle.sin();
                let dz = dist * angle.cos();
                let dy = rng.random_range(-VERTICAL_RANGE..=VERTICAL_RANGE);
                candidates.push((dx, dy, dz));
            }
            candidates
        };

        let threat_to_mob_sq = threat_pos.squared_distance_to_vec(&mob_pos);

        for (dx, dy, dz) in candidates {
            if dx.abs() > HORIZONTAL_RANGE || dz.abs() > HORIZONTAL_RANGE {
                continue;
            }

            let candidate = BlockPos::new(
                (mob_pos.x + dx) as i32,
                (mob_pos.y + dy as f64) as i32,
                (mob_pos.z + dz) as i32,
            );

            let block_at = world.get_block_state(&candidate).await;
            let block_below = world
                .get_block_state(&BlockPos::new(
                    candidate.0.x,
                    candidate.0.y - 1,
                    candidate.0.z,
                ))
                .await;

            if block_at.is_solid() || !block_below.is_solid() {
                continue;
            }

            let flee_vec = Vector3::new(
                candidate.0.x as f64 + 0.5,
                candidate.0.y as f64,
                candidate.0.z as f64 + 0.5,
            );

            if threat_pos.squared_distance_to_vec(&flee_vec) < threat_to_mob_sq {
                continue;
            }

            return Some(flee_vec);
        }

        None
    }
}

impl Goal for AvoidEntityGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let threat = self.find_threat(mob);
            let Some(target) = threat else {
                return false;
            };

            let threat_pos = target.get_entity().pos.load();
            let flee_pos = Self::find_flee_position(mob, &threat_pos).await;
            let Some(pos) = flee_pos else {
                return false;
            };

            self.target = Some(target);
            self.flee_pos = Some(pos);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let navigator = mob.get_mob_entity().navigator.lock().await;
            !navigator.is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(flee_pos) = self.flee_pos {
                let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal::new(mob_pos, flee_pos, self.slow_speed));
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = &self.target {
                let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let threat_pos = target.get_entity().pos.load();
                let dist_sq = mob_pos.squared_distance_to_vec(&threat_pos);
                let speed = if dist_sq < FAST_DISTANCE_SQ {
                    self.fast_speed
                } else {
                    self.slow_speed
                };
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_speed(speed);
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
            self.flee_pos = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

use std::sync::Arc;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{EntityBase, ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;

const FAST_DISTANCE_SQ: f64 = 49.0;
const FLEE_VECTOR_LEN: f64 = 16.0;

pub struct AvoidEntityGoal {
    goal_control: Controls,
    flee_type: &'static EntityType,
    flee_distance: f64,
    slow_speed: f64,
    fast_speed: f64,
    target: Option<Arc<dyn EntityBase>>,
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

    fn compute_flee_pos(mob_pos: &Vector3<f64>, threat_pos: &Vector3<f64>) -> Vector3<f64> {
        let dx = mob_pos.x - threat_pos.x;
        let dz = mob_pos.z - threat_pos.z;
        let len = dx.hypot(dz).max(0.001);
        Vector3::new(
            mob_pos.x + dx / len * FLEE_VECTOR_LEN,
            mob_pos.y,
            mob_pos.z + dz / len * FLEE_VECTOR_LEN,
        )
    }
}

impl Goal for AvoidEntityGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.target = self.find_threat(mob);
            self.target.is_some()
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
            if let Some(target) = &self.target {
                let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let threat_pos = target.get_entity().pos.load();
                let dist_sq = mob_pos.squared_distance_to_vec(&threat_pos);
                let speed = if dist_sq < FAST_DISTANCE_SQ {
                    self.fast_speed
                } else {
                    self.slow_speed
                };
                let flee_pos = Self::compute_flee_pos(&mob_pos, &threat_pos);
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal::new(mob_pos, flee_pos, speed));
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

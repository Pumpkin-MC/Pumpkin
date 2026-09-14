use super::{Controls, Goal, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use std::sync::Arc;

pub struct FollowMobGoal {
    goal_control: Controls,
    following_mob: Option<Arc<dyn EntityBase>>,
    speed_modifier: f64,
    time_to_recalc_path: i32,
    stop_distance_sq: f64,
    area_size: f64,
}

impl FollowMobGoal {
    #[must_use]
    pub fn new(speed_modifier: f64, stop_distance: f32, area_size: f32) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            following_mob: None,
            speed_modifier,
            time_to_recalc_path: 0,
            stop_distance_sq: f64::from(stop_distance * stop_distance),
            area_size: f64::from(area_size),
        }
    }
}

impl Goal for FollowMobGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let my_type = mob.get_entity().entity_type;
        let my_uuid = mob.get_entity().entity_uuid;
        let pos = mob.get_entity().pos.load();
        let world = mob.get_entity().world.load();

        let nearby = world.get_nearby_entities(pos, self.area_size);
        for candidate in nearby.values() {
            let c_entity = candidate.get_entity();
            if c_entity.entity_uuid == my_uuid {
                continue;
            }
            if c_entity.entity_type == my_type {
                continue;
            }
            if candidate.get_living_entity().is_some() {
                self.following_mob = Some(candidate.clone());
                return true;
            }
        }

        false
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let Some(following) = &self.following_mob else {
            return false;
        };

        if !following.get_entity().is_alive() {
            return false;
        }

        let is_idle = mob
            .get_mob_entity()
            .navigator
            .try_lock()
            .is_ok_and(|nav| nav.is_idle());

        let mob_pos = mob.get_entity().pos.load();
        let follow_pos = following.get_entity().pos.load();
        let dist_sq = mob_pos.squared_distance_to_vec(&follow_pos);

        !is_idle && dist_sq > self.stop_distance_sq
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.time_to_recalc_path = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.following_mob = None;
        let mut navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.stop();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(following) = &self.following_mob else {
            return;
        };

        let mob_entity = mob.get_mob_entity();
        let follow_pos = following.get_entity().pos.load();
        let mob_pos = mob.get_entity().pos.load();

        mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity(mob, following);

        self.time_to_recalc_path -= 1;
        if self.time_to_recalc_path <= 0 {
            self.time_to_recalc_path = to_goal_ticks(10);
            let dist_sq = mob_pos.squared_distance_to_vec(&follow_pos);

            let mut navigator = mob_entity
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            if dist_sq > self.stop_distance_sq {
                navigator.set_progress(NavigatorGoal::new(
                    mob_pos,
                    follow_pos,
                    self.speed_modifier,
                ));
            } else {
                navigator.stop();
            }
        }
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

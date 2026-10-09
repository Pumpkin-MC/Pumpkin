use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{Controls, Goal};
use crate::entity::ai::util::land_random_pos;
use crate::entity::mob::Mob;

/// Mirrors vanilla Minecraft's `net.minecraft.world.entity.ai.goal.StrollThroughVillageGoal`.
pub struct StrollThroughVillageGoal {
    goal_control: Controls,
    pub speed: f64,
    pub interval: i32,
    pub search_radius: i32,
    pub target_pos: Option<Vector3<f64>>,
}

impl StrollThroughVillageGoal {
    #[must_use]
    pub const fn new(speed: f64, interval: i32) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            interval,
            search_radius: 32,
            target_pos: None,
        }
    }

    #[must_use]
    pub const fn with_search_radius(speed: f64, interval: i32, search_radius: i32) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            interval,
            search_radius,
            target_pos: None,
        }
    }
}

impl Goal for StrollThroughVillageGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let is_day = (world.get_time_of_day() % 24000) < 12000;
        if is_day {
            return false;
        }

        if rand::rng().random_range(0..self.interval) != 0 {
            return false;
        }

        let mob_pos = entity.pos.load();
        let entities = world.entities.load();
        let radius_sq = f64::from(self.search_radius) * f64::from(self.search_radius);
        let near_village = entities.iter().any(|e| {
            e.get_entity().entity_type == &EntityType::VILLAGER
                && mob_pos.squared_distance_to_vec(&e.get_entity().pos.load()) <= radius_sq
        });

        if !near_village {
            return false;
        }

        if let Some(target) = land_random_pos::get_pos(mob, 15, 7) {
            self.target_pos = Some(target);
            true
        } else {
            false
        }
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.is_navigator_idle() && self.target_pos.is_some()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target_pos {
            let mob_pos = mob.get_entity().pos.load();
            mob.navigate_to(mob_pos, target, self.speed);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target_pos = None;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::poi::SectionPos;
use rand::RngExt;

use super::{Controls, Goal};
use crate::entity::ai::util::land_random_pos;
use crate::entity::mob::Mob;
use crate::world::World;

/// Checks whether a block position is within range of a village.
#[must_use]
pub fn is_near_village(world: &World, block_pos: BlockPos, search_distance: i32) -> bool {
    world.is_close_to_village(&block_pos, search_distance)
}

/// Mirrors vanilla Minecraft's `net.minecraft.world.entity.ai.goal.StrollThroughVillageGoal`.
pub struct StrollThroughVillageGoal {
    goal_control: Controls,
    pub speed: f64,
    pub interval: i32,
    pub search_distance: i32,
    pub target_pos: Option<Vector3<f64>>,
}

impl StrollThroughVillageGoal {
    #[must_use]
    pub const fn new(speed: f64, interval: i32) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            interval,
            search_distance: 6,
            target_pos: None,
        }
    }

    #[must_use]
    pub const fn with_search_distance(speed: f64, interval: i32, search_distance: i32) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            interval,
            search_distance,
            target_pos: None,
        }
    }

    #[must_use]
    pub const fn with_search_radius(speed: f64, interval: i32, search_distance: i32) -> Self {
        Self::with_search_distance(speed, interval, search_distance)
    }
}

impl Goal for StrollThroughVillageGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        if world.is_bright_outside() {
            return false;
        }

        if rand::rng().random_range(0..self.interval.max(1)) != 0 {
            return false;
        }

        let block_pos = entity.block_pos.load();
        if !world.is_close_to_village(&block_pos, self.search_distance) {
            return false;
        }

        if let Some(target) = land_random_pos::get_pos_weighted(mob, 15, 7, |px| {
            f64::from(-world.sections_to_village(SectionPos::of(&px)))
        }) {
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

    fn stop(&mut self, mob: &dyn Mob) {
        self.target_pos = None;
        mob.stop_navigation();
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

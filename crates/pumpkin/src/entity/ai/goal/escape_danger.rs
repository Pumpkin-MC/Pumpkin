use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal};
use crate::entity::ai::goal::try_find_water::TryFindWaterGoal;
use crate::entity::ai::util::default_random_pos;
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};

const WATER_SEARCH_XZ: i32 = 5;
const WATER_SEARCH_Y: i32 = 1;
const HORIZONTAL_RANGE: i32 = 5;
const VERTICAL_RANGE: i32 = 4;

pub struct EscapeDangerGoal {
    speed: f64,
    goal_control: Controls,
    target: Option<Vector3<f64>>,
    running: bool,
}

impl EscapeDangerGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            speed,
            goal_control: Controls::MOVE,
            target: None,
            running: false,
        })
    }

    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.running
    }

    /// Only `#minecraft:panic_causes` damage makes a mob flee, and only while the source is still
    /// remembered.
    fn should_panic(mob: &dyn Mob) -> bool {
        mob.get_mob_entity()
            .living_entity
            .get_last_damage_type()
            .is_some_and(|damage_type| {
                damage_type.has_tag(&tag::DamageType::MINECRAFT_PANIC_CAUSES)
            })
    }

    /// Nearest water within 5 blocks horizontally, only when the mob is not stuck in a block.
    fn look_for_water(mob: &dyn Mob) -> Option<BlockPos> {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let mob_pos = entity.block_pos.load();

        if world.get_block_state(&mob_pos).is_solid() {
            return None;
        }

        let mut best: Option<(i32, BlockPos)> = None;
        for dx in -WATER_SEARCH_XZ..=WATER_SEARCH_XZ {
            for dy in -WATER_SEARCH_Y..=WATER_SEARCH_Y {
                for dz in -WATER_SEARCH_XZ..=WATER_SEARCH_XZ {
                    let pos = BlockPos::new(mob_pos.0.x + dx, mob_pos.0.y + dy, mob_pos.0.z + dz);
                    if !TryFindWaterGoal::is_water(&world, &pos) {
                        continue;
                    }
                    let distance = dx.abs() + dy.abs() + dz.abs();
                    if best.is_none_or(|(best_distance, _)| distance < best_distance) {
                        best = Some((distance, pos));
                    }
                }
            }
        }

        best.map(|(_, pos)| pos)
    }

    fn find_random_position(mob: &dyn Mob) -> Option<Vector3<f64>> {
        default_random_pos::get_pos(mob, HORIZONTAL_RANGE, VERTICAL_RANGE)
    }
}

impl Goal for EscapeDangerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !Self::should_panic(mob) {
            return false;
        }

        if mob.get_entity().fire_ticks.load(Relaxed) > 0
            && let Some(water) = Self::look_for_water(mob)
        {
            self.target = Some(water.to_f64());
            return true;
        }

        self.target = Self::find_random_position(mob);
        self.target.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            let pos = mob.get_mob_entity().living_entity.entity.pos.load();
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_progress(NavigatorGoal::new(pos, target, self.speed));
        }
        self.running = true;
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
        self.running = false;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

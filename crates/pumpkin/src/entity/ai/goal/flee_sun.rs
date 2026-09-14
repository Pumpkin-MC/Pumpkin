use super::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct FleeSunGoal {
    goal_control: Controls,
    speed_modifier: f64,
    wanted_x: f64,
    wanted_y: f64,
    wanted_z: f64,
}

impl FleeSunGoal {
    #[must_use]
    pub const fn new(speed_modifier: f64) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed_modifier,
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
        }
    }

    fn find_hide_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let mob_pos = mob.get_entity().block_pos.load();
        let world = mob.get_entity().world.load();
        let mut rng = mob.get_random();

        for _ in 0..10 {
            let offset_x = rng.random_range(-10..=10);
            let offset_y = rng.random_range(-3..=3);
            let offset_z = rng.random_range(-10..=10);
            let check_pos = BlockPos::new(
                mob_pos.0.x + offset_x,
                mob_pos.0.y + offset_y,
                mob_pos.0.z + offset_z,
            );

            let block_at = world.get_block_state(&check_pos);
            let block_above = world.get_block_state(&check_pos.up());
            let block_below = world.get_block_state(&check_pos.down());

            if !block_at.is_solid() && !block_above.is_solid() && block_below.is_solid() {
                return Some(Vector3::new(
                    f64::from(check_pos.0.x) + 0.5,
                    f64::from(check_pos.0.y),
                    f64::from(check_pos.0.z) + 0.5,
                ));
            }
        }

        None
    }
}

impl Goal for FleeSunGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_mob_entity().get_target().is_some() {
            return false;
        }

        if !mob.get_mob_entity().living_entity.entity.is_on_fire() {
            return false;
        }

        let has_helmet = mob
            .get_mob_entity()
            .living_entity
            .entity_equipment
            .try_lock()
            .is_ok_and(|eq| !eq.get(&EquipmentSlot::HEAD).is_empty());

        if has_helmet {
            return false;
        }

        if let Some(pos) = Self::find_hide_pos(mob) {
            self.wanted_x = pos.x;
            self.wanted_y = pos.y;
            self.wanted_z = pos.z;
            true
        } else {
            false
        }
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let is_idle = mob
            .get_mob_entity()
            .navigator
            .try_lock()
            .is_ok_and(|nav| nav.is_idle());
        !is_idle
    }

    fn start(&mut self, mob: &dyn Mob) {
        let mob_pos = mob.get_entity().pos.load();
        let mut navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_progress(NavigatorGoal::new(
            mob_pos,
            Vector3::new(self.wanted_x, self.wanted_y, self.wanted_z),
            self.speed_modifier,
        ));
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

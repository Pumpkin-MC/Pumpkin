use super::{Controls, Goal};
use crate::entity::mob::Mob;
use pumpkin_data::data_component_impl::EquipmentSlot;

#[derive(Default)]
pub struct RestrictSunGoal;

impl RestrictSunGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Goal for RestrictSunGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let has_helmet = mob
            .get_mob_entity()
            .living_entity
            .entity_equipment
            .try_lock()
            .is_ok_and(|eq| !eq.get(&EquipmentSlot::HEAD).is_empty());

        if has_helmet {
            return false;
        }

        let time = mob.get_entity().world.load().get_world_age() % 24000;
        time < 12000
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let has_helmet = mob
            .get_mob_entity()
            .living_entity
            .entity_equipment
            .try_lock()
            .is_ok_and(|eq| !eq.get(&EquipmentSlot::HEAD).is_empty());

        if has_helmet {
            return false;
        }

        let time = mob.get_entity().world.load().get_world_age() % 24000;
        time < 12000
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}

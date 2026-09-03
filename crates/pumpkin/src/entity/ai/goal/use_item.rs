use super::{Controls, Goal};
use crate::entity::mob::Mob;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_util::Hand;
use std::sync::Arc;

pub type CanUsePredicate = Arc<dyn Fn(&dyn Mob) -> bool + Send + Sync>;

pub struct UseItemGoal {
    goal_control: Controls,
    item: ItemStack,
    finish_sound: Option<Sound>,
    can_use_fn: CanUsePredicate,
}

impl UseItemGoal {
    #[must_use]
    pub fn new(item: ItemStack, finish_sound: Option<Sound>, can_use_fn: CanUsePredicate) -> Self {
        Self {
            goal_control: Controls::empty(),
            item,
            finish_sound,
            can_use_fn,
        }
    }
}

impl Goal for UseItemGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        (self.can_use_fn)(mob)
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        mob.get_mob_entity()
            .living_entity
            .active_hand
            .lock()
            .is_ok_and(|hand| hand.is_some())
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Ok(mut eq) = mob
            .get_mob_entity()
            .living_entity
            .entity_equipment
            .try_lock()
        {
            eq.put(&EquipmentSlot::MAIN_HAND, self.item.clone());
        }
        mob.get_mob_entity().living_entity.set_active_hand(
            Hand::Right,
            self.item.clone(),
            i32::MAX,
        );
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Ok(mut eq) = mob
            .get_mob_entity()
            .living_entity
            .entity_equipment
            .try_lock()
        {
            eq.put(&EquipmentSlot::MAIN_HAND, ItemStack::EMPTY.clone());
        }
        mob.get_mob_entity().living_entity.clear_active_hand();
        if let Some(sound) = self.finish_sound {
            let pos = mob.get_entity().pos.load();
            mob.get_entity().world.load().play_sound(
                sound,
                pumpkin_data::sound::SoundCategory::Hostile,
                &pos,
            );
        }
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

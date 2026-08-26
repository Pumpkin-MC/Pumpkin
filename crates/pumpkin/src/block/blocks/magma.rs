use std::sync::atomic::Ordering;

use pumpkin_data::{
    Enchantment, damage::DamageType, data_component_impl::EquipmentSlot, effect::StatusEffect,
};
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, OnEntityStepArgs};

#[pumpkin_block("minecraft:magma_block")]
pub struct MagmaBlock;

impl BlockBehaviour for MagmaBlock {
    fn on_entity_step(&self, args: OnEntityStepArgs<'_>) {
        {
            // Only living entities take damage
            let Some(living_entity) = args.entity.get_living_entity() else {
                return;
            };

            let ent = args.entity.get_entity();

            // Don't damage if sneaking
            if ent.is_sneaking() {
                return;
            }

            // Fire immune entities don't take damage
            if ent.entity_type.fire_immune || ent.fire_immune.load(Ordering::Relaxed) {
                return;
            }

            let has_frost_walker = {
                let equipment = living_entity
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                equipment
                    .equipment
                    .get(&EquipmentSlot::FEET)
                    .is_some_and(|boots| {
                        boots.get_enchantment_level(&Enchantment::FROST_WALKER) != 0
                    })
            };
            if has_frost_walker {
                return;
            }

            if living_entity
                .get_effect(&StatusEffect::FIRE_RESISTANCE)
                .is_some()
            {
                return;
            }

            // Apply damage
            args.entity.damage(args.entity, 1.0, DamageType::HOT_FLOOR);
        }
    }
}

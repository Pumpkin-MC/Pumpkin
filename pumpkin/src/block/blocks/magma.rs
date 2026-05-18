use std::sync::atomic::Ordering;

use pumpkin_data::{
    Enchantment, damage::DamageType, data_component_impl::EquipmentSlot, effect::StatusEffect,
};
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, BlockFuture, OnEntityStepArgs};

#[pumpkin_block("minecraft:magma_block")]
pub struct MagmaBlock;

impl BlockBehaviour for MagmaBlock {
    fn on_entity_step<'a>(&'a self, args: OnEntityStepArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Only living entities take damage
            let Some(living_entity) = args.entity.get_living_entity() else {
                return;
            };

            let ent = args.entity.get_entity();

            // Don't damage if sneaking
            if ent.sneaking.load(Ordering::Relaxed) {
                return;
            }

            // Fire immune entities don't take damage
            if ent.entity_type.fire_immune || ent.fire_immune.load(Ordering::Relaxed) {
                return;
            }

            let has_frost_walker = {
                let equipment = living_entity.entity_equipment.lock().await;
                let boots = equipment.get(&EquipmentSlot::FEET);
                let boots_stack = boots.lock().await;
                boots_stack.get_enchantment_level(&Enchantment::FROST_WALKER) != 0
            };
            if has_frost_walker {
                return;
            }

            if living_entity
                .get_effect(&StatusEffect::FIRE_RESISTANCE)
                .await
                .is_some()
            {
                return;
            }

            // Apply damage
            args.entity
                .damage(args.entity, 1.0, DamageType::HOT_FLOOR)
                .await;
        })
    }
}

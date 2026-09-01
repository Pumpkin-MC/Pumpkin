use pumpkin_data::{
    Block, BlockDirection, BlockStateId, Enchantment, block_properties::CampfireLikeProperties,
    damage::DamageType, data_component_impl::EquipmentSlot, effect::StatusEffect, fluid::Fluid,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::tick::TickPriority;

use crate::block::entities::campfire::CampfireBlockEntity;
use crate::{
    block::{
        BlockBehaviour, BlockIsReplacing, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs,
        OnPlaceArgs, PlacedArgs,
    },
    entity::EntityBase,
};
use std::sync::Arc;

#[pumpkin_block_from_tag("minecraft:campfires")]
pub struct CampfireBlock;

impl BlockBehaviour for CampfireBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let entity = CampfireBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        }
    }

    // TODO: cooking food on campfire (CampfireBlockEntity)
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        {
            if CampfireLikeProperties::from_state_id(args.state.id).lit
                && let Some(living_entity) = args.entity.get_living_entity()
            {
                let has_frost_walker_enchantment = {
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
                let has_fire_res = living_entity
                    .get_effect(&StatusEffect::FIRE_RESISTANCE)
                    .is_some();
                if has_frost_walker_enchantment || has_fire_res {
                    //campfire burning doesn't work if entity's boots has frost walker enchantment or entity has fire resistance. source: https://minecraft.wiki/w/Campfire#Damage
                    return;
                }
                let damage_amount = if args.block == &Block::SOUL_CAMPFIRE {
                    2.0
                } else {
                    1.0
                };
                args.entity
                    .damage(args.entity, damage_amount, DamageType::CAMPFIRE);
            }
        }
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let is_replacing_water = matches!(args.replacing, BlockIsReplacing::Water(_));
        let mut props = CampfireLikeProperties::from_state_id(args.block.default_state.id);
        props.waterlogged = is_replacing_water;
        props.signal_fire = is_signal_fire_base_block(args.world.get_block(&args.position.down()));
        props.lit = !is_replacing_water;
        props.facing = args.player.get_entity().get_horizontal_facing();
        props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let mut props = CampfireLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            props.lit = false;
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }

        if args.direction == BlockDirection::Down {
            props.signal_fire =
                is_signal_fire_base_block(args.world.get_block(args.neighbor_position));
        }

        props.to_state_id(args.block)
    }

    // TODO: onProjectileHit
}

fn is_signal_fire_base_block(block: &Block) -> bool {
    block == &Block::HAY_BLOCK
}

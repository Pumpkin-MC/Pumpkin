use pumpkin_data::{
    Block, BlockDirection, Enchantment,
    block_properties::{BlockProperties, CampfireLikeProperties},
    damage::DamageType,
    data_component_impl::EquipmentSlot,
    effect::StatusEffect,
    fluid::Fluid,
};
use pumpkin_world::{BlockStateId, tick::TickPriority};

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockIsReplacing, BlockMetadata,
        GetStateForNeighborUpdateArgs, OnEntityCollisionArgs, OnPlaceArgs,
    },
    entity::EntityBase,
};

pub struct CampfireBlock;

impl BlockMetadata for CampfireBlock {
    fn ids() -> Box<[u16]> {
        [Block::CAMPFIRE.id, Block::SOUL_CAMPFIRE.id].into()
    }
}

impl BlockBehaviour for CampfireBlock {
    // TODO: cooking food on campfire (CampfireBlockEntity)
    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if CampfireLikeProperties::from_state_id(args.state.id, args.block).lit
                && let Some(living_entity) = args.entity.get_living_entity()
            {
                let has_frost_walker_enchantment = {
                    let equipment = living_entity.entity_equipment.lock().await;
                    let boots = equipment.get(&EquipmentSlot::FEET);

                    let boots_stack = boots.lock().await;

                    boots_stack.get_enchantment_level(&Enchantment::FROST_WALKER) != 0
                };
                let has_fire_res = living_entity
                    .get_effect(&StatusEffect::FIRE_RESISTANCE)
                    .await
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
                    .damage(args.entity, damage_amount, DamageType::CAMPFIRE)
                    .await;
            }
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let is_replacing_water = matches!(args.replacing, BlockIsReplacing::Water(_));
            let mut props =
                CampfireLikeProperties::from_state_id(args.block.default_state.id, args.block);
            props.waterlogged = is_replacing_water;
            props.signal_fire =
                is_signal_fire_base_block(args.world.get_block(&args.position.down()).await);
            props.lit = !is_replacing_water;
            props.facing = args.player.get_entity().get_horizontal_facing();
            props.to_state_id(args.block)
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = CampfireLikeProperties::from_state_id(args.state_id, args.block);
            if props.waterlogged {
                props.lit = false;
                args.world
                    .schedule_fluid_tick(
                        &Fluid::WATER,
                        *args.position,
                        Fluid::WATER.flow_speed as u8,
                        TickPriority::Normal,
                    )
                    .await;
            }

            if args.direction == BlockDirection::Down {
                props.signal_fire =
                    is_signal_fire_base_block(args.world.get_block(args.neighbor_position).await);
            }

            props.to_state_id(args.block)
        })
    }

    // TODO: onProjectileHit
}

fn is_signal_fire_base_block(block: &Block) -> bool {
    block == &Block::HAY_BLOCK
}

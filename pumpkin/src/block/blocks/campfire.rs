use pumpkin_data::{
    Block, BlockDirection, BlockStateId, Enchantment,
    block_properties::{BlockProperties, CampfireLikeProperties},
    damage::DamageType,
    data_component_impl::EquipmentSlot,
    effect::StatusEffect,
    fluid::Fluid,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::campfire::CampfireBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockIsReplacing, GetStateForNeighborUpdateArgs,
        NormalUseArgs, OnEntityCollisionArgs, OnPlaceArgs, OnProjectileHitArgs, PlacedArgs,
        UseWithItemArgs,
    },
    entity::EntityBase,
};
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[pumpkin_block_from_tag("minecraft:campfires")]
pub struct CampfireBlock;

impl BlockBehaviour for CampfireBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let Some(block_entity) = args.world.get_block_entity(args.position) else {
                return BlockActionResult::Pass;
            };
            let Some(inventory) = block_entity.get_inventory() else {
                return BlockActionResult::Pass;
            };
            for slot in 0..inventory.size() {
                let item = inventory.remove_stack(slot).await;
                if !item.is_empty() {
                    args.player
                        .inventory
                        .offer_or_drop_stack(item, args.player.as_ref())
                        .await;
                    args.player
                        .increment_stat(
                            pumpkin_data::statistic::StatisticCategory::Custom,
                            pumpkin_data::statistic::CustomStatistic::InteractWithCampfire as i32,
                            1,
                        )
                        .await;
                    return BlockActionResult::Success;
                }
            }
            BlockActionResult::Pass
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let Some(block_entity) = args.world.get_block_entity(args.position) else {
                return BlockActionResult::Pass;
            };
            let Some(campfire) = block_entity.as_any().downcast_ref::<CampfireBlockEntity>() else {
                return BlockActionResult::Pass;
            };
            let mut item = args.item_stack.lock().await;
            if campfire
                .add_item(&mut item, args.player.is_creative())
                .await
            {
                args.player
                    .increment_stat(
                        pumpkin_data::statistic::StatisticCategory::Custom,
                        pumpkin_data::statistic::CustomStatistic::InteractWithCampfire as i32,
                        1,
                    )
                    .await;
                BlockActionResult::Success
            } else {
                BlockActionResult::Pass
            }
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = CampfireBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        })
    }

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
                is_signal_fire_base_block(args.world.get_block(&args.position.down()));
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
        })
    }

    fn on_projectile_hit<'a>(&'a self, args: OnProjectileHitArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let props = CampfireLikeProperties::from_state_id(args.state.id, args.block);
            if args
                .projectile
                .get_entity()
                .fire_ticks
                .load(Ordering::Relaxed)
                > 0
                && !props.lit
                && !props.waterlogged
            {
                let mut lit_props = props;
                lit_props.lit = true;
                args.world
                    .set_block_state(
                        args.position,
                        lit_props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        })
    }
}

fn is_signal_fire_base_block(block: &Block) -> bool {
    block == &Block::HAY_BLOCK
}

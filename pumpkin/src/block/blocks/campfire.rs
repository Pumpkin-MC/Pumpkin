use pumpkin_data::{
    Block, BlockDirection, BlockStateId, Enchantment,
    block_properties::{BlockProperties, CampfireLikeProperties},
    damage::DamageType,
    data_component_impl::EquipmentSlot,
    effect::StatusEffect,
    fluid::Fluid,
    recipes::{CookingRecipeKind, get_cooking_recipe_with_ingredient},
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::tick::TickPriority;

use crate::block::entities::campfire::CampfireBlockEntity;
use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockIsReplacing, BlockActionResult,
        GetStateForNeighborUpdateArgs, NormalUseArgs, OnEntityCollisionArgs, OnPlaceArgs,
        PlacedArgs, RandomTickArgs, UseWithItemArgs,
    },
    entity::EntityBase,
};
use pumpkin_world::inventory::Inventory;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[pumpkin_block_from_tag("minecraft:campfires")]
pub struct CampfireBlock;

impl BlockBehaviour for CampfireBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = CampfireBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if !args.server.basic_config.allow_campfire_manual_pickup {
                return BlockActionResult::Pass;
            }

            if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(campfire) =
                    block_entity.as_any().downcast_ref::<CampfireBlockEntity>()
            {
                for slot in 0..4 {
                    let is_finished =
                        campfire.cooking_times[slot].load(Ordering::Relaxed)
                            >= campfire.cooking_total_times[slot].load(Ordering::Relaxed)
                            && !campfire.items[slot].lock().await.is_empty();

                    if is_finished {
                        let result = campfire.remove_stack(slot).await;
                        campfire.cooking_times[slot].store(0, Ordering::Relaxed);
                        campfire.cooking_total_times[slot].store(0, Ordering::Relaxed);
                        campfire.dirty.store(true, Ordering::Relaxed);

                        args.world.update_block_entity(&block_entity);

                        args.player
                            .inventory()
                            .offer_or_drop_stack(result, args.player.as_ref())
                            .await;

                        return BlockActionResult::Success;
                    }
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
            let recipe = {
                let guard = args.item_stack.lock().await;
                get_cooking_recipe_with_ingredient(guard.item, CookingRecipeKind::CampfireCooking)
            };
            let Some(recipe) = recipe else {
                return BlockActionResult::PassToDefaultBlockAction;
            };

            if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(campfire) =
                    block_entity.as_any().downcast_ref::<CampfireBlockEntity>()
            {
                let empty_slot = {
                    let mut found = None;
                    for i in 0..4 {
                        if campfire.items[i].lock().await.is_empty() {
                            found = Some(i);
                            break;
                        }
                    }
                    found
                };

                if let Some(slot) = empty_slot {
                    let placed_item = {
                        let mut guard = args.item_stack.lock().await;
                        guard.split_unless_creative(args.player.gamemode.load(), 1)
                    };

                    campfire
                        .cooking_total_times[slot]
                        .store(recipe.cookingtime, Ordering::Relaxed);
                    campfire.set_stack(slot, placed_item).await;

                    args.world.update_block_entity(&block_entity);

                    return BlockActionResult::Success;
                }
            }
            BlockActionResult::PassToDefaultBlockAction
        })
    }

    fn random_tick<'a>(&'a self, _args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {})
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

    // TODO: onProjectileHit
}

fn is_signal_fire_base_block(block: &Block) -> bool {
    block == &Block::HAY_BLOCK
}

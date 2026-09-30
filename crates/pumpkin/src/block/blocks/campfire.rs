use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId, Enchantment,
    block_properties::CampfireLikeProperties,
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
        BlockBehaviour, BlockIsReplacing, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs,
        OnPlaceArgs, OnProjectileHitArgs, PathComputationType, PlacedArgs, UseWithItemArgs,
        registry::BlockActionResult,
    },
    entity::EntityBase,
};
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

#[pumpkin_block_from_tag("minecraft:campfires")]
pub struct CampfireBlock;

impl BlockBehaviour for CampfireBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = CampfireBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.position);
        if !CampfireLikeProperties::from_state_id(state.id).lit {
            return BlockActionResult::PassToDefaultBlockAction;
        }

        let Some(recipe) = get_cooking_recipe_with_ingredient(
            args.item_stack.item,
            CookingRecipeKind::CampfireCooking,
        ) else {
            return BlockActionResult::PassToDefaultBlockAction;
        };

        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return BlockActionResult::PassToDefaultBlockAction;
        };
        let Some(campfire) = block_entity.as_any().downcast_ref::<CampfireBlockEntity>() else {
            return BlockActionResult::PassToDefaultBlockAction;
        };

        for slot in 0..CampfireBlockEntity::SLOT_COUNT {
            let mut stored = campfire.items[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !stored.is_empty() {
                continue;
            }

            *stored = args
                .item_stack
                .split_unless_creative(args.player.gamemode.load(), 1);
            *campfire.cooking_times[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
            *campfire.cooking_total_times[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = recipe.cookingtime;
            drop(stored);

            args.player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::InteractWithCampfire as i32,
                1,
            );
            args.world.update_block_entity(&block_entity);
            return BlockActionResult::Success;
        }

        BlockActionResult::PassToDefaultBlockAction
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
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
                // Campfire damage is prevented by Frost Walker boots or fire resistance.
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

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }

    fn on_projectile_hit(&self, args: OnProjectileHitArgs<'_>) {
        let Some(new_state_id) = relit_state_id(
            args.block,
            args.state.id,
            args.projectile.get_entity().is_on_fire(),
        ) else {
            return;
        };

        args.world
            .set_block_state(args.position, new_state_id, BlockFlags::NOTIFY_ALL);
    }
}

fn relit_state_id(
    block: &Block,
    state_id: BlockStateId,
    projectile_on_fire: bool,
) -> Option<BlockStateId> {
    if !projectile_on_fire {
        return None;
    }

    let props = CampfireLikeProperties::from_state_id(state_id);
    if props.lit || props.waterlogged {
        return None;
    }

    let mut relit = props;
    relit.lit = true;
    Some(relit.to_state_id(block))
}

fn is_signal_fire_base_block(block: &Block) -> bool {
    block == &Block::HAY_BLOCK
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::block_properties::HorizontalFacing;

    fn campfire_state(block: &Block, lit: bool, waterlogged: bool) -> BlockStateId {
        let mut props = CampfireLikeProperties::default(block);
        props.lit = lit;
        props.waterlogged = waterlogged;
        props.to_state_id(block)
    }

    #[test]
    fn burning_projectile_relights_unlit_campfire() {
        for block in [&Block::CAMPFIRE, &Block::SOUL_CAMPFIRE] {
            let state_id = campfire_state(block, false, false);
            let new_state_id =
                relit_state_id(block, state_id, true).expect("campfire should relight");

            assert!(CampfireLikeProperties::from_state_id(new_state_id).lit);
            assert_eq!(Block::from_state_id(new_state_id), block);
        }
    }

    #[test]
    fn relighting_keeps_facing_and_signal_fire() {
        let mut props = CampfireLikeProperties::default(&Block::CAMPFIRE);
        props.lit = false;
        props.signal_fire = true;
        props.facing = HorizontalFacing::West;
        let state_id = props.to_state_id(&Block::CAMPFIRE);

        let new_state_id =
            relit_state_id(&Block::CAMPFIRE, state_id, true).expect("campfire should relight");
        let new_props = CampfireLikeProperties::from_state_id(new_state_id);

        assert!(new_props.lit);
        assert!(new_props.signal_fire);
        assert_eq!(new_props.facing, HorizontalFacing::West);
    }

    #[test]
    fn campfire_does_not_relight_unless_unlit_and_dry() {
        let waterlogged = campfire_state(&Block::CAMPFIRE, false, true);
        assert!(relit_state_id(&Block::CAMPFIRE, waterlogged, true).is_none());

        let lit = campfire_state(&Block::CAMPFIRE, true, false);
        assert!(relit_state_id(&Block::CAMPFIRE, lit, true).is_none());

        let unlit = campfire_state(&Block::CAMPFIRE, false, false);
        assert!(relit_state_id(&Block::CAMPFIRE, unlit, false).is_none());
    }
}

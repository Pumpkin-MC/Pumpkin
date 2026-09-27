use std::sync::Arc;

use crate::{
    block::{
        BlockBehaviour, GetComparatorOutputArgs, NormalUseArgs, OnScheduledTickArgs,
        PathComputationType, UseWithItemArgs, registry::BlockActionResult,
    },
    entity::{Entity, item::ItemEntity},
    world::World,
};
use pumpkin_data::{
    Block, BlockState, BlockStateId,
    block_properties::ComposterLikeProperties,
    data_component_impl::CompostableImpl,
    entity::EntityType,
    item::Item,
    item_stack::ItemStack,
    sound::{Sound, SoundCategory},
    world::WorldEvent,
};
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{tick::TickPriority, world::BlockFlags};
use rand::RngExt;

/// Maximum composter level indicating bone meal is ready for extraction.
const READY_FOR_HARVEST_LEVEL: u8 = 8;

/// Composter level indicating that composting is in progress and waiting for the final maturation tick.
const COMPOSTING_READY_LEVEL: u8 = 7;

/// Delay in ticks before level 7 transitions to level 8 (ready for harvest).
const MATURATION_DELAY_TICKS: u8 = 20;

#[pumpkin_block("minecraft:composter")]
pub struct ComposterBlock;

impl BlockBehaviour for ComposterBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let props = ComposterLikeProperties::from_state_id(state_id);
        if props.level == READY_FOR_HARVEST_LEVEL {
            self.clear_composter(args.world, args.position, state_id, args.block);
            return BlockActionResult::SuccessServer;
        }

        BlockActionResult::Pass
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let props = ComposterLikeProperties::from_state_id(state_id);
        let level = props.level;

        // Check if the composter is full
        if level == READY_FOR_HARVEST_LEVEL {
            self.clear_composter(args.world, args.position, state_id, args.block);
            return BlockActionResult::SuccessServer;
        }

        let item_stack = &mut *args.item_stack;
        let Some(compostable) = item_stack.get_data_component::<CompostableImpl>() else {
            return BlockActionResult::Pass;
        };
        let chance = compostable.chance;

        // Consume one item from the stack (if in survival mode)
        if !args.player.has_infinite_materials() {
            item_stack.decrement(1);
        }

        // Determine if the composter level should increase
        let level_increased = level < COMPOSTING_READY_LEVEL
            && (level == 0 || rand::rng().random_bool(f64::from(chance)));
        if level_increased {
            self.update_level_composter(args.world, args.position, state_id, args.block, level + 1);
        }
        args.world.sync_world_event(
            WorldEvent::ComposterFill,
            *args.position,
            i32::from(level_increased),
        );

        BlockActionResult::SuccessServer
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let props = ComposterLikeProperties::from_state_id(state_id);
        let level = props.level;
        if level == COMPOSTING_READY_LEVEL {
            self.update_level_composter(args.world, args.position, state_id, args.block, level + 1);
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        {
            let props = ComposterLikeProperties::from_state_id(args.state.id);
            Some(props.level)
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

impl ComposterBlock {
    pub fn update_level_composter(
        &self,
        world: &Arc<World>,
        location: &BlockPos,
        state_id: BlockStateId,
        block: &Block,
        level: u8,
    ) {
        let mut props = ComposterLikeProperties::from_state_id(state_id);
        props.level = level;
        world.set_block_state(location, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
        if level == COMPOSTING_READY_LEVEL {
            world.schedule_block_tick(
                block,
                *location,
                MATURATION_DELAY_TICKS,
                TickPriority::Normal,
            );
        }
    }

    pub fn clear_composter(
        &self,
        world: &Arc<World>,
        location: &BlockPos,
        state_id: BlockStateId,
        block: &Block,
    ) {
        self.update_level_composter(world, location, state_id, block, 0);

        let item_position = {
            let mut rng = rand::rng();
            location.to_centered_f64().add_raw(
                rng.random_range(-0.35..=0.35),
                rng.random_range(-0.35..=0.35) + 0.51,
                rng.random_range(-0.35..=0.35),
            )
        };

        let item_entity = ItemEntity::new(
            Entity::new(world.clone(), item_position, &EntityType::ITEM),
            ItemStack::new(1, &Item::BONE_MEAL),
        );

        world.play_block_sound(Sound::BlockComposterEmpty, SoundCategory::Blocks, *location);

        world.spawn_entity(Arc::new(item_entity));
    }
}

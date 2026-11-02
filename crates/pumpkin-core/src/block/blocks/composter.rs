use std::sync::Arc;

use crate::{
    block::{
        BlockBehaviour, GetComparatorOutputArgs, NormalUseArgs, OnScheduledTickArgs,
        PathComputationType, PlacedArgs, UseWithItemArgs, registry::BlockActionResult,
    },
    entity::{Entity, item::ItemEntity},
    world::World,
};
use pumpkin_data::{
    Block, BlockState, BlockStateId,
    block_properties::ComposterLikeProperties,
    data_component_impl::CompostableImpl,
    entity::EntityType,
    game_event::GameEvent,
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

#[pumpkin_block("minecraft:composter")]
pub struct ComposterBlock;

impl BlockBehaviour for ComposterBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        if ComposterLikeProperties::from_state_id(args.state_id).level == 7 {
            args.world
                .schedule_block_tick(args.block, *args.position, 20, TickPriority::Normal);
        }
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let props = ComposterLikeProperties::from_state_id(state_id);
        if props.level == 8 {
            self.clear_composter(args.world, args.position, state_id, args.block);
            return BlockActionResult::Success;
        }

        BlockActionResult::Pass
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let props = ComposterLikeProperties::from_state_id(state_id);
        let level = props.level;

        let item_stack = &mut *args.item_stack;
        if let Some(compostable) = item_stack.get_data_component::<CompostableImpl>()
            && level < 8
        {
            if level < 7 {
                let chance = compostable.chance;
                if !args.player.has_infinite_materials() {
                    item_stack.decrement(1);
                }

                let increased = level == 0 || rand::rng().random_bool(f64::from(chance));
                if increased {
                    self.update_level_composter(
                        args.world,
                        args.position,
                        state_id,
                        args.block,
                        level + 1,
                    );
                }
                args.world.sync_world_event(
                    WorldEvent::ComposterFill,
                    *args.position,
                    i32::from(increased),
                );
            }
            return BlockActionResult::Success;
        }

        BlockActionResult::PassToDefaultBlockAction
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let props = ComposterLikeProperties::from_state_id(state_id);
        let level = props.level;
        if level == 7 {
            self.update_level_composter(args.world, args.position, state_id, args.block, level + 1);
            args.world.play_sound_fine(
                Sound::BlockComposterReady,
                SoundCategory::Blocks,
                &args.position.to_centered_f64(),
                1.0,
                1.0,
            );
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        let props = ComposterLikeProperties::from_state_id(args.state.id);
        Some(props.level)
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
        world.emit_game_event(GameEvent::BlockChange.name(), location.to_centered_f64());
        if level == 7 {
            world.schedule_block_tick(block, *location, 20, TickPriority::Normal);
        }
    }

    pub fn clear_composter(
        &self,
        world: &Arc<World>,
        location: &BlockPos,
        state_id: BlockStateId,
        block: &Block,
    ) {
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

        world.spawn_entity(Arc::new(item_entity));
        self.update_level_composter(world, location, state_id, block, 0);
        world.play_sound_fine(
            Sound::BlockComposterEmpty,
            SoundCategory::Blocks,
            &location.to_centered_f64(),
            1.0,
            1.0,
        );
    }
}

use std::sync::Arc;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, GetComparatorOutputArgs, NormalUseArgs, OnScheduledTickArgs,
        UseWithItemArgs, registry::BlockActionResult,
    },
    entity::{Entity, item::ItemEntity},
    world::World,
};
use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{BlockProperties, ComposterLikeProperties},
    composter_increase_chance::get_composter_increase_chance_from_item_id,
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

#[pumpkin_block("minecraft:composter")]
pub struct ComposterBlock;

impl BlockBehaviour for ComposterBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position);
            let props = ComposterLikeProperties::from_state_id(state_id, args.block);
            if props.level == 8 {
                self.clear_composter(args.world, args.position, state_id, args.block)
                    .await;
            }

            BlockActionResult::Pass
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position);
            let props = ComposterLikeProperties::from_state_id(state_id, args.block);
            let level = props.level;

            // Check if the composter is full
            if level == 8 {
                self.clear_composter(args.world, args.position, state_id, args.block)
                    .await;
                return BlockActionResult::Consume;
            }

            let mut item_stack = args.item_stack.lock().await;
            let item_id = item_stack.item.id;

            // Check if the item is consumable by the composter
            let Some(chance) = get_composter_increase_chance_from_item_id(item_id) else {
                return BlockActionResult::Pass;
            };

            // Consume one item from the stack (if in survival mode). Vanilla only
            // consumes below the "full" level (7); at 7 the interaction is a no-op
            // until the composter is emptied.
            if level < 7 && !args.player.has_infinite_materials() {
                item_stack.decrement(1);
            }

            // Determine if the composter level should increase
            if level < 7 {
                let rose = level == 0 || rand::rng().random_bool(f64::from(chance));
                if rose {
                    self.update_level_composter(
                        args.world,
                        args.position,
                        state_id,
                        args.block,
                        level + 1,
                    )
                    .await;
                }
                // levelEvent(1500, pos, state != newState ? 1 : 0): vanilla fires this
                // for every accepted item, using data 0 for the "did not rise" variant.
                args.world.sync_world_event(
                    WorldEvent::ComposterFill,
                    *args.position,
                    i32::from(rose),
                );
            }

            // Consume the item
            BlockActionResult::Consume
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position);
            let props = ComposterLikeProperties::from_state_id(state_id, args.block);
            let level = props.level;
            if level == 7 {
                self.update_level_composter(
                    args.world,
                    args.position,
                    state_id,
                    args.block,
                    level + 1,
                )
                .await;
                args.world.play_sound(
                    Sound::BlockComposterReady,
                    SoundCategory::Blocks,
                    &args.position.to_centered_f64(),
                );
            }
        })
    }

    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            let props = ComposterLikeProperties::from_state_id(args.state.id, args.block);
            Some(props.level)
        })
    }
}

impl ComposterBlock {
    pub async fn update_level_composter(
        &self,
        world: &Arc<World>,
        location: &BlockPos,
        state_id: BlockStateId,
        block: &Block,
        level: u8,
    ) {
        let mut props = ComposterLikeProperties::from_state_id(state_id, block);
        props.level = level;
        world
            .set_block_state(location, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
            .await;
        if level == 7 {
            world.schedule_block_tick(block, *location, 20, TickPriority::Normal);
        }
    }

    pub async fn clear_composter(
        &self,
        world: &Arc<World>,
        location: &BlockPos,
        state_id: BlockStateId,
        block: &Block,
    ) {
        self.update_level_composter(world, location, state_id, block, 0)
            .await;

        // Vec3.atLowerCornerWithOffset(pos, 0.5, 1.01, 0.5).offsetRandomXZ(random, 0.7F)
        // jitters X and Z only; Y is exactly pos.y + 1.01.
        let item_position = {
            let mut rng = rand::rng();
            location.to_centered_f64().add_raw(
                rng.random_range(-0.35..=0.35),
                0.51,
                rng.random_range(-0.35..=0.35),
            )
        };

        let item_entity = ItemEntity::new(
            Entity::new(world.clone(), item_position, &EntityType::ITEM),
            ItemStack::new(1, &Item::BONE_MEAL),
        );

        world.spawn_entity(Arc::new(item_entity)).await;

        world.play_sound(
            Sound::BlockComposterEmpty,
            SoundCategory::Blocks,
            &location.to_centered_f64(),
        );
    }
}

use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockMetadata, GetComparatorOutputArgs, UseWithItemArgs};
use pumpkin_data::Block;
use pumpkin_data::BlockId;
use pumpkin_data::block_properties::{BlockProperties, WaterCauldronLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_world::world::BlockFlags;

pub struct CauldronBlock;

impl BlockMetadata for CauldronBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::CAULDRON,
            BlockId::WATER_CAULDRON,
            BlockId::LAVA_CAULDRON,
            BlockId::POWDER_SNOW_CAULDRON,
        ]
        .into()
    }
}

fn fire_cauldron_change(
    world: &std::sync::Arc<crate::world::World>,
    pos: pumpkin_util::math::position::BlockPos,
    old_level: i32,
    new_level: i32,
    reason: crate::plugin::block::cauldron_level_change::CauldronChangeReason,
    entity: Option<std::sync::Arc<dyn crate::entity::EntityBase>>,
) -> bool {
    let mut event = crate::plugin::block::cauldron_level_change::CauldronLevelChangeEvent {
        block_pos: pos,
        world: world.clone(),
        old_level,
        new_level,
        reason,
        entity,
        cancelled: false,
    };
    if let Some(server) = world.server.upgrade() {
        server.plugin_manager.fire_blocking(&server, &mut event);
    }
    !event.cancelled
}

fn give_item_or_drop(
    player: &crate::entity::player::Player,
    world: &std::sync::Arc<crate::world::World>,
    item: &'static Item,
) {
    let mut stack = ItemStack::new(1, item);
    let was_added = player.inventory.insert_stack_anywhere(&mut stack);
    if !was_added && !stack.is_empty() {
        world.drop_stack(&player.position().to_block_pos(), stack);
    }
}

impl BlockBehaviour for CauldronBlock {
    #[allow(clippy::too_many_lines)]
    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let item_id = args.item_stack.item.id;
        let block_id = args.block.id;
        let gamemode = args.player.gamemode.load();

        // Filling empty cauldron with buckets
        if block_id == BlockId::CAULDRON {
            if item_id == Item::WATER_BUCKET.id {
                if !fire_cauldron_change(
                    args.world,
                    *args.position,
                    0,
                    3,
                    crate::plugin::block::cauldron_level_change::CauldronChangeReason::BucketEmpty,
                    Some(Arc::clone(args.player) as Arc<dyn crate::entity::EntityBase>),
                ) {
                    return BlockActionResult::Pass;
                }
                let state_id = Block::WATER_CAULDRON
                    .from_properties(&[("level", "3")])
                    .to_state_id(&Block::WATER_CAULDRON);
                args.world
                    .set_block_state(args.position, state_id, BlockFlags::NOTIFY_ALL);
                args.world.play_sound(
                    Sound::ItemBucketEmpty,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                args.item_stack.decrement_unless_creative(gamemode, 1);
                give_item_or_drop(args.player, args.world, &Item::BUCKET);
                return BlockActionResult::Success;
            } else if item_id == Item::LAVA_BUCKET.id {
                args.world.set_block_state(
                    args.position,
                    Block::LAVA_CAULDRON.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
                args.world.play_sound(
                    Sound::ItemBucketEmptyLava,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                args.item_stack.decrement_unless_creative(gamemode, 1);
                give_item_or_drop(args.player, args.world, &Item::BUCKET);
                return BlockActionResult::Success;
            } else if item_id == Item::POWDER_SNOW_BUCKET.id {
                let state_id = Block::POWDER_SNOW_CAULDRON
                    .from_properties(&[("level", "3")])
                    .to_state_id(&Block::POWDER_SNOW_CAULDRON);
                args.world
                    .set_block_state(args.position, state_id, BlockFlags::NOTIFY_ALL);
                args.world.play_sound(
                    Sound::ItemBucketEmptyPowderSnow,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                args.item_stack.decrement_unless_creative(gamemode, 1);
                give_item_or_drop(args.player, args.world, &Item::BUCKET);
                return BlockActionResult::Success;
            } else if item_id == Item::POTION.id {
                let state_id = Block::WATER_CAULDRON
                    .from_properties(&[("level", "1")])
                    .to_state_id(&Block::WATER_CAULDRON);
                args.world
                    .set_block_state(args.position, state_id, BlockFlags::NOTIFY_ALL);
                args.world.play_sound(
                    Sound::ItemBottleEmpty,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                args.item_stack.decrement_unless_creative(gamemode, 1);
                give_item_or_drop(args.player, args.world, &Item::GLASS_BOTTLE);
                return BlockActionResult::Success;
            }
        }

        // Collecting fluid from full cauldrons into empty bucket
        if item_id == Item::BUCKET.id {
            let state_id = args.world.get_block_state_id(args.position);
            let (filled_item, sound) = if block_id == BlockId::WATER_CAULDRON {
                let props = WaterCauldronLikeProperties::from_state_id(state_id, args.block);
                if props.level == 3 {
                    (Some(&Item::WATER_BUCKET), Sound::ItemBucketFill)
                } else {
                    (None, Sound::ItemBucketFill)
                }
            } else if block_id == BlockId::LAVA_CAULDRON {
                (Some(&Item::LAVA_BUCKET), Sound::ItemBucketFillLava)
            } else if block_id == BlockId::POWDER_SNOW_CAULDRON {
                let props = WaterCauldronLikeProperties::from_state_id(state_id, args.block);
                if props.level == 3 {
                    (
                        Some(&Item::POWDER_SNOW_BUCKET),
                        Sound::ItemBucketFillPowderSnow,
                    )
                } else {
                    (None, Sound::ItemBucketFillPowderSnow)
                }
            } else {
                (None, Sound::ItemBucketFill)
            };

            if let Some(result_item) = filled_item {
                args.world.set_block_state(
                    args.position,
                    Block::CAULDRON.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
                args.world
                    .play_sound(sound, SoundCategory::Blocks, &args.position.to_f64());
                args.item_stack.decrement_unless_creative(gamemode, 1);
                give_item_or_drop(args.player, args.world, result_item);
                return BlockActionResult::Success;
            }
        }

        // Adding water bottle to non-full water cauldron
        if block_id == BlockId::WATER_CAULDRON && item_id == Item::POTION.id {
            let state_id = args.world.get_block_state_id(args.position);
            let props = WaterCauldronLikeProperties::from_state_id(state_id, args.block);
            if props.level < 3 {
                let next_level_str = match props.level {
                    1 => "2",
                    _ => "3",
                };
                let new_state_id = Block::WATER_CAULDRON
                    .from_properties(&[("level", next_level_str)])
                    .to_state_id(&Block::WATER_CAULDRON);
                args.world
                    .set_block_state(args.position, new_state_id, BlockFlags::NOTIFY_ALL);
                args.world.play_sound(
                    Sound::ItemBottleEmpty,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                args.item_stack.decrement_unless_creative(gamemode, 1);
                give_item_or_drop(args.player, args.world, &Item::GLASS_BOTTLE);
                return BlockActionResult::Success;
            }
        }

        BlockActionResult::PassToDefaultBlockAction
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        match args.block.id {
            BlockId::WATER_CAULDRON | BlockId::POWDER_SNOW_CAULDRON => {
                let state_id = args.world.get_block_state_id(args.position);
                let props = WaterCauldronLikeProperties::from_state_id(state_id, args.block);
                Some(props.level)
            }
            BlockId::LAVA_CAULDRON => Some(3),
            _ => Some(0),
        }
    }
}

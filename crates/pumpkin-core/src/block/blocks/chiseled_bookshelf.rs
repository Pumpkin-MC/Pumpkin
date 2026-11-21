use std::sync::{Arc, atomic::Ordering};

use pumpkin_macros::pumpkin_block;

use crate::block::entities::chiseled_bookshelf::ChiseledBookshelfBlockEntity;
use crate::{
    block::{
        BlockBehaviour, GetComparatorOutputArgs, NormalUseArgs, OnPlaceArgs, PlacedArgs,
        UseWithItemArgs, registry::BlockActionResult,
    },
    entity::{EntityBase, player::Player},
    world::World,
};
use pumpkin_data::{
    BlockStateId,
    block_properties::ChiseledBookshelfLikeProperties,
    game_event::GameEvent,
    item::Item,
    item_stack::ItemStack,
    sound::{Sound, SoundCategory},
    statistic::StatisticCategory,
    tag,
    tag::Taggable,
};
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_util::math::position::BlockPos;

#[pumpkin_block("minecraft:chiseled_bookshelf")]
pub struct ChiseledBookshelfBlock;

impl ChiseledBookshelfBlock {
    const ROWS: usize = 2;
    const COLUMNS: usize = 3;
}

impl BlockBehaviour for ChiseledBookshelfBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut properties = ChiseledBookshelfLikeProperties::default(args.block);

        // Face in the opposite direction the player is facing
        properties.facing = args.player.get_entity().get_horizontal_facing().opposite();

        properties.to_state_id(args.block)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.position);
        let properties = ChiseledBookshelfLikeProperties::from_state_id(state.id);

        if let Some(slot) = args
            .hit
            .get_hit_slot(properties.facing, Self::ROWS, Self::COLUMNS)
        {
            if Self::is_slot_used(properties, slot) {
                if let Some(block_entity) = args.world.get_block_entity(args.position)
                    && let Some(block_entity) = block_entity
                        .as_any()
                        .downcast_ref::<ChiseledBookshelfBlockEntity>()
                {
                    Self::try_remove_book(
                        args.world,
                        args.player,
                        args.position,
                        block_entity,
                        properties,
                        slot,
                    );
                    return BlockActionResult::Success;
                }
            } else {
                return BlockActionResult::Consume;
            }
        }
        BlockActionResult::Pass
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.position);
        let properties = ChiseledBookshelfLikeProperties::from_state_id(state.id);

        if !args
            .item_stack
            .get_item()
            .has_tag(&tag::Item::MINECRAFT_BOOKSHELF_BOOKS)
        {
            return BlockActionResult::PassToDefaultBlockAction;
        }
        if let Some(slot) = args
            .hit
            .get_hit_slot(properties.facing, Self::ROWS, Self::COLUMNS)
        {
            if Self::is_slot_used(properties, slot) {
                return BlockActionResult::PassToDefaultBlockAction;
            } else if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(block_entity) = block_entity
                    .as_any()
                    .downcast_ref::<ChiseledBookshelfBlockEntity>()
            {
                Self::try_add_book(
                    args.world,
                    args.player,
                    args.position,
                    block_entity,
                    properties,
                    slot,
                    args.item_stack,
                );
                return BlockActionResult::Success;
            }
        }

        BlockActionResult::Pass
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let block_entity = ChiseledBookshelfBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(block_entity));
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(block_entity) = block_entity
                .as_any()
                .downcast_ref::<ChiseledBookshelfBlockEntity>()
        {
            return Some((block_entity.last_interacted_slot.load(Ordering::Relaxed) + 1) as u8);
        }
        None
    }
}

impl ChiseledBookshelfBlock {
    fn try_add_book(
        world: &Arc<World>,
        player: &Player,
        position: &BlockPos,
        entity: &ChiseledBookshelfBlockEntity,
        properties: ChiseledBookshelfLikeProperties,
        slot: usize,
        item: &mut ItemStack,
    ) {
        let used_item_id = item.item.id;
        player.increment_stat(StatisticCategory::Used, i32::from(used_item_id), 1);

        let sound = if item.get_item() == &Item::ENCHANTED_BOOK {
            Sound::BlockChiseledBookshelfInsertEnchanted
        } else {
            Sound::BlockChiseledBookshelfInsert
        };

        entity.set_book(slot, item.split_unless_creative(player.gamemode.load(), 1));
        entity.update_state(properties, world, slot);

        world.play_sound(sound, SoundCategory::Blocks, &position.to_centered_f64());
    }

    fn try_remove_book(
        world: &Arc<World>,
        player: &Arc<Player>,
        position: &BlockPos,
        entity: &ChiseledBookshelfBlockEntity,
        properties: ChiseledBookshelfLikeProperties,
        slot: usize,
    ) {
        let mut stack = entity.remove_book(slot, 1);

        let sound = if stack.get_item() == &Item::ENCHANTED_BOOK {
            Sound::BlockChiseledBookshelfPickupEnchanted
        } else {
            Sound::BlockChiseledBookshelfPickup
        };

        if player.get_inventory().insert_stack_anywhere(&mut stack) {
            player.sync_inventory_to_client();
        }
        if !stack.is_empty() {
            player.drop_item(stack);
        }
        entity.update_state(properties, world, slot);

        world.play_sound(sound, SoundCategory::Blocks, &position.to_centered_f64());
        world.emit_game_event(GameEvent::BlockChange.name(), position.to_centered_f64());
    }

    const fn is_slot_used(properties: ChiseledBookshelfLikeProperties, slot: usize) -> bool {
        match slot {
            0 => properties.slot_0_occupied,
            1 => properties.slot_1_occupied,
            2 => properties.slot_2_occupied,
            3 => properties.slot_3_occupied,
            4 => properties.slot_4_occupied,
            5 => properties.slot_5_occupied,
            _ => false,
        }
    }
}

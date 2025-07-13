use std::sync::{Arc, atomic::Ordering};

use async_trait::async_trait;
use pumpkin_data::{
    block_properties::{BlockProperties, ChiseledBookshelfLikeProperties, HorizontalFacing},
    item::Item,
    sound::{Sound, SoundCategory},
    tag::Tagable,
};
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use pumpkin_world::{
    BlockStateId, block::entities::chiseled_bookshelf::ChiseledBookshelfBlockEntity,
    inventory::Inventory, item::ItemStack,
};
use tokio::sync::Mutex;

use crate::{
    block::{
        pumpkin_block::{
            BlockHitResult, GetComparatorOutputArgs, NormalUseArgs, OnPlaceArgs,
            OnStateReplacedArgs, PlacedArgs, PumpkinBlock, UseWithItemArgs,
        },
        registry::BlockActionResult,
    },
    entity::{EntityBase, player::Player},
    world::World,
};

#[pumpkin_block("minecraft:chiseled_bookshelf")]
pub struct ChiseledBookshelfBlock;

#[async_trait]
impl PumpkinBlock for ChiseledBookshelfBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut properties = ChiseledBookshelfLikeProperties::default(args.block);

        // Face in the opposite direction the player is facing
        properties.facing = args.player.get_entity().get_horizontal_facing().opposite();

        properties.to_state_id(args.block)
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.position).await;
        let properties = ChiseledBookshelfLikeProperties::from_state_id(state.id, args.block);

        if let Some(slot) = Self::get_slot_for_hit(args.hit, properties.facing) {
            if Self::is_slot_used(properties, slot) {
                if let Some(block_entity) = args.world.get_block_entity(args.position).await {
                    if let Some(block_entity) = block_entity
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
                        )
                        .await;
                        return BlockActionResult::Success;
                    }
                }
            } else {
                return BlockActionResult::Consume;
            }
        }
        BlockActionResult::Pass
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.position).await;
        let properties = ChiseledBookshelfLikeProperties::from_state_id(state.id, args.block);

        if !args
            .item_stack
            .lock()
            .await
            .get_item()
            .is_tagged_with("minecraft:bookshelf_books")
            .unwrap_or(false)
        {
            return BlockActionResult::PassToDefaultBlockAction;
        }
        if let Some(slot) = Self::get_slot_for_hit(args.hit, properties.facing) {
            if Self::is_slot_used(properties, slot) {
                return BlockActionResult::PassToDefaultBlockAction;
            } else if let Some(block_entity) = args.world.get_block_entity(args.position).await {
                if let Some(block_entity) = block_entity
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
                    )
                    .await;
                    return BlockActionResult::Success;
                }
            }
        }

        BlockActionResult::Pass
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let block_entity = ChiseledBookshelfBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(block_entity)).await;
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world.remove_block_entity(args.position).await;
    }

    async fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        if let Some(block_entity) = args.world.get_block_entity(args.position).await {
            if let Some(block_entity) = block_entity
                .as_any()
                .downcast_ref::<ChiseledBookshelfBlockEntity>()
            {
                return Some((block_entity.last_interacted_slot.load(Ordering::Relaxed) + 1) as u8);
            }
        }
        None
    }
}

impl ChiseledBookshelfBlock {
    async fn try_add_book(
        world: &Arc<World>,
        player: &Player,
        position: &BlockPos,
        entity: &ChiseledBookshelfBlockEntity,
        properties: ChiseledBookshelfLikeProperties,
        slot: i8,
        item: &Arc<Mutex<ItemStack>>,
    ) {
        // TODO: Increment used stats for chiseled bookshelf on the player

        let mut item = item.lock().await;
        let sound = if *item.get_item() == Item::ENCHANTED_BOOK {
            Sound::BlockChiseledBookshelfPickupEnchanted
        } else {
            Sound::BlockChiseledBookshelfPickup
        };

        entity
            .set_stack(
                slot as usize,
                item.split_unless_creative(player.gamemode.load(), 1),
            )
            .await;
        entity.update_state(properties, world.clone(), slot).await;

        world
            .play_sound(sound, SoundCategory::Blocks, &position.to_centered_f64())
            .await;
    }

    async fn try_remove_book(
        world: &Arc<World>,
        player: &Player,
        position: &BlockPos,
        entity: &ChiseledBookshelfBlockEntity,
        properties: ChiseledBookshelfLikeProperties,
        slot: i8,
    ) {
        let mut stack = entity.remove_stack_specific(slot as usize, 1).await;

        let sound = if *stack.get_item() == Item::ENCHANTED_BOOK {
            Sound::BlockChiseledBookshelfPickupEnchanted
        } else {
            Sound::BlockChiseledBookshelfPickup
        };

        if !player
            .get_inventory()
            .insert_stack_anywhere(&mut stack)
            .await
        {
            // Drop the item on the ground if the player cannot hold it because of a full inventory
            player.drop_item(stack).await;
        }
        entity.update_state(properties, world.clone(), slot).await;

        world
            .play_sound(sound, SoundCategory::Blocks, &position.to_centered_f64())
            .await;
    }

    fn get_slot_for_hit(hit: &BlockHitResult<'_>, facing: HorizontalFacing) -> Option<i8> {
        Self::get_hit_pos(hit, facing).map(|position| {
            let i = i8::from(position.y < 0.5);
            let j = Self::get_column(position.x);
            j + i * 3
        })
    }

    fn get_hit_pos(hit: &BlockHitResult<'_>, facing: HorizontalFacing) -> Option<Vector2<f32>> {
        // If the direction is not horizontal, we cannot hit a slot
        let direction = hit.side.to_horizontal_facing()?;

        // If the facing direction does not match the block's facing, we cannot hit a slot
        if facing != direction {
            return None;
        }

        match direction {
            HorizontalFacing::North => Some(Vector2::new(1.0 - hit.cursor_pos.x, hit.cursor_pos.y)),
            HorizontalFacing::South => Some(Vector2::new(hit.cursor_pos.x, hit.cursor_pos.y)),
            HorizontalFacing::West => Some(Vector2::new(hit.cursor_pos.z, hit.cursor_pos.y)),
            HorizontalFacing::East => Some(Vector2::new(1.0 - hit.cursor_pos.z, hit.cursor_pos.y)),
        }
    }

    // Magic numbers for the slots
    // These are based on the vanilla chiseled bookshelf implementation
    const OFFSET_SLOT_0: f32 = 0.375;
    const OFFSET_SLOT_1: f32 = 0.6875;

    fn get_column(x: f32) -> i8 {
        if x < Self::OFFSET_SLOT_0 {
            0
        } else if x < Self::OFFSET_SLOT_1 {
            1
        } else {
            2
        }
    }

    fn is_slot_used(properties: ChiseledBookshelfLikeProperties, slot: i8) -> bool {
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

use async_trait::async_trait;
use log::info;
use pumpkin_data::block_properties::{
    BlockProperties, ChiseledBookshelfLikeProperties, HorizontalFacing,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::BlockStateId;

use crate::{
    block::{
        pumpkin_block::{
            BlockHitResult, NormalUseArgs, OnPlaceArgs, PumpkinBlock, UseWithItemArgs,
        },
        registry::BlockActionResult,
    },
    entity::EntityBase,
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

    async fn normal_use(&self, args: NormalUseArgs<'_>) {
        info!("Position: {:?}", args.location);
        info!("Cursor position: {:?}", args.hit.cursor_pos);

        let state = args.world.get_block_state(args.location).await;
        let props = ChiseledBookshelfLikeProperties::from_state_id(state.id, args.block);

        let slot = self.get_slot_for_hit(&args.hit, &props.facing).await;

        info!("Hit slot: {:?}", slot);
    }

    async fn use_with_item(&self, _args: UseWithItemArgs<'_>) -> BlockActionResult {
        BlockActionResult::Continue
    }
}

impl ChiseledBookshelfBlock {
    async fn get_slot_for_hit(
        &self,
        hit: &BlockHitResult<'_>,
        facing: &HorizontalFacing,
    ) -> Option<i32> {
        if let Some(position) = Self::get_hit_pos(hit, facing) {
            let i = if position.z >= 0.5 { 0 } else { 1 };
            let j = Self::get_column(position.x);
            Some(j + i * 3)
        } else {
            None
        }
    }

    fn get_hit_pos(hit: &BlockHitResult<'_>, facing: &HorizontalFacing) -> Option<Vector2<f32>> {
        // If the direction is not horizontal, we cannot hit a slot
        let Some(direction) = hit.side.to_horizontal_facing() else {
            return None;
        };

        // If the facing direction does not match the block's facing, we cannot hit a slot
        if *facing != direction {
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

    fn get_column(x: f32) -> i32 {
        if x < Self::OFFSET_SLOT_0 {
            0
        } else if x < Self::OFFSET_SLOT_1 {
            1
        } else {
            2
        }
    }
}

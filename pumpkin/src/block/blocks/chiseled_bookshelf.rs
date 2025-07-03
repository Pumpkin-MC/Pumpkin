use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, ChiseledBookshelfLikeProperties};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

use crate::{block::{pumpkin_block::{NormalUseArgs, OnPlaceArgs, PumpkinBlock, UseWithItemArgs}, registry::BlockActionResult}, entity::EntityBase};

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

    async fn normal_use(&self, _args: NormalUseArgs<'_>) {}

    async fn use_with_item(&self, _args: UseWithItemArgs<'_>) -> BlockActionResult {
        BlockActionResult::Continue
    }
}

impl ChiseledBookshelfBlock {}

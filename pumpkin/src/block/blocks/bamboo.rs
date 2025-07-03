use async_trait::async_trait;
use pumpkin_data::tag::Tagable;
use pumpkin_macros::pumpkin_block;

use crate::block::pumpkin_block::{CanPlaceAtArgs, PumpkinBlock};

#[pumpkin_block("minecraft:bamboo")]
pub struct BambooBlock;

#[async_trait]
impl PumpkinBlock for BambooBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.location.down()).await;
        block_below
            .is_tagged_with("minecraft:bamboo_plantable_on")
            .unwrap()
    }
}

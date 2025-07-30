use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, CanPlaceAtArgs};

#[pumpkin_block("minecraft:vine")]
pub struct VineBlock;

#[async_trait]
impl BlockBehaviour for VineBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        // TODO: This is bad and not vanilla, just a "hotfix"
        for dir in BlockDirection::all() {
            if !args
                .block_accessor
                .get_block_state(&args.position.offset(dir.to_offset()))
                .await
                .is_air()
            {
                return true;
            }
        }
        false
    }
}

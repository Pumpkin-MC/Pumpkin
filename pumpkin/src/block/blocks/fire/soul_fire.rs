use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::tag::Tagable;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

use crate::block::pumpkin_block::{
    BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, PumpkinBlock,
};

use super::FireBlockBase;

#[pumpkin_block("minecraft:soul_fire")]
pub struct SoulFireBlock;

impl SoulFireBlock {
    #[must_use]
    pub fn is_soul_base(block: &Block) -> bool {
        block
            .is_tagged_with("minecraft:soul_fire_base_blocks")
            .unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for SoulFireBlock {
    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !Self::is_soul_base(args.world.get_block(&args.location.down()).await) {
            return Block::AIR.default_state.id;
        }

        args.state_id
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::is_soul_base(args.block_accessor.get_block(&args.location.down()).await)
    }

    async fn broken(&self, args: BrokenArgs<'_>) {
        FireBlockBase::broken(args.world.clone(), *args.location).await;
    }
}

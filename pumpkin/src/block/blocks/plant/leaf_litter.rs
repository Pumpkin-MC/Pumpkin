use async_trait::async_trait;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_world::BlockStateId;

use crate::block::pumpkin_block::{
    BlockMetadata, CanPlaceAtArgs, CanUpdateAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    PumpkinBlock,
};

use super::segmented::Segmented;

type LeafLitterProperties = pumpkin_data::block_properties::LeafLitterLikeProperties;

pub struct LeafLitterBlock;

impl BlockMetadata for LeafLitterBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &["leaf_litter"]
    }
}

#[async_trait]
impl PumpkinBlock for LeafLitterBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args
            .block_accessor
            .get_block_state(&args.location.down())
            .await;
        block_below.is_side_solid(BlockDirection::Up)
    }

    async fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        Segmented::can_update_at(self, args).await
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        Segmented::on_place(self, args).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Down {
            let block_below_state = args.world.get_block_state(&args.location.down()).await;
            if !block_below_state.is_side_solid(BlockDirection::Up) {
                return Block::AIR.default_state.id;
            }
        }
        args.state_id
    }
}

impl Segmented for LeafLitterBlock {
    type Properties = LeafLitterProperties;
}

use pumpkin_data::{Block, BlockDirection};
use pumpkin_world::BlockStateId;

use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, CanUpdateAtArgs,
    GetStateForNeighborUpdateArgs, OnPlaceArgs,
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

impl BlockBehaviour for LeafLitterBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let block_below = args
                .block_accessor
                .get_block_state(&args.position.down())
                .await;
            block_below.is_side_solid(BlockDirection::Up)
        })
    }

    fn can_update_at<'a>(&'a self, args: CanUpdateAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move { Segmented::can_update_at(self, args).await })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move { Segmented::on_place(self, args).await })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.direction == BlockDirection::Down {
                let block_below_state = args.world.get_block_state(&args.position.down()).await;
                if !block_below_state.is_side_solid(BlockDirection::Up) {
                    return Block::AIR.default_state.id;
                }
            }
            args.state_id
        })
    }
}

impl Segmented for LeafLitterBlock {
    type Properties = LeafLitterProperties;
}

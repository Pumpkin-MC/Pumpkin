use pumpkin_data::BlockStateId;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockId, tag};

use crate::block::blocks::plant::PlantBlockBase;

use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, CanUpdateAtArgs,
    GetStateForNeighborUpdateArgs, OnPlaceArgs,
};

use super::segmented::Segmented;

type FlowerbedProperties = pumpkin_data::block_properties::PinkPetalsLikeProperties;

pub struct FlowerbedBlock;

impl BlockMetadata for FlowerbedBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::PINK_PETALS, BlockId::WILDFLOWERS].into()
    }
}

impl BlockBehaviour for FlowerbedBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.position.down());
        block_below.has_tag(&tag::Block::MINECRAFT_DIRT) || block_below == &Block::FARMLAND
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        Segmented::can_update_at(self, args)
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Segmented::on_place(self, args)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
            .await
        })
    }
}

impl PlantBlockBase for FlowerbedBlock {}

impl Segmented for FlowerbedBlock {
    type Properties = FlowerbedProperties;
}

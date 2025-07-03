use async_trait::async_trait;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_world::BlockStateId;

use crate::block::pumpkin_block::{
    BlockMetadata, CanPlaceAtArgs, CanUpdateAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    PumpkinBlock,
};

use super::segmented::Segmented;

type FlowerbedProperties = pumpkin_data::block_properties::PinkPetalsLikeProperties;

pub struct FlowerbedBlock;

impl BlockMetadata for FlowerbedBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &["pink_petals", "wildflowers"]
    }
}

#[async_trait]
impl PumpkinBlock for FlowerbedBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.location.down()).await;
        block_below.is_tagged_with("minecraft:dirt").unwrap() || block_below == &Block::FARMLAND
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
            let block_below = args.world.get_block(&args.location.down()).await;
            if !(block_below.is_tagged_with("minecraft:dirt").unwrap()
                || block_below == &Block::FARMLAND)
            {
                return Block::AIR.default_state.id;
            }
        }
        args.state_id
    }
}

impl Segmented for FlowerbedBlock {
    type Properties = FlowerbedProperties;
}

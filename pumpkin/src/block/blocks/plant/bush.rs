use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_world::BlockStateId;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::pumpkin_block::{
    BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, PumpkinBlock,
};

pub struct BushBlock;

impl BlockMetadata for BushBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::BUSH.name, Block::FIREFLY_BUSH.name]
    }
}

#[async_trait]
impl PumpkinBlock for BushBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
        .await
    }
}

impl PlantBlockBase for BushBlock {}

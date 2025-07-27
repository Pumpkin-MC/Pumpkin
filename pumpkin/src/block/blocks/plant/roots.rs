use async_trait::async_trait;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

use crate::block::{
    blocks::plant::PlantBlockBase,
    pumpkin_block::{BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, PumpkinBlock},
};

pub struct RootsBlock;

impl BlockMetadata for RootsBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::WARPED_ROOTS.name, Block::CRIMSON_ROOTS.name]
    }
}

#[async_trait]
impl PumpkinBlock for RootsBlock {
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

impl PlantBlockBase for RootsBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block_below = block_accessor.get_block(pos).await;
        block_below.is_tagged_with_by_tag(&tag::Block::MINECRAFT_NYLIUM)
            || block_below == &Block::SOUL_SOIL
            || block_below.is_tagged_with_by_tag(&tag::Block::MINECRAFT_DIRT)
            || block_below == &Block::FARMLAND
    }

    async fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        self.can_plant_on_top(block_accessor, &block_pos.down())
            .await
    }
}

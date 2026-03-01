use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs};
use crate::block::{GetStateForNeighborUpdateArgs, blocks::plant::PlantBlockBase};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockAccessor;
pub struct FungusBlock;

impl BlockMetadata for FungusBlock {
    fn ids() -> Box<[u16]> {
        [Block::CRIMSON_FUNGUS.id, Block::WARPED_FUNGUS.id].into()
    }
}

impl BlockBehaviour for FungusBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
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
impl PlantBlockBase for FungusBlock {
    async fn can_plant_on_top(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        let block = block_accessor.get_block(pos).await;
        supports_fungus(block)
    }
    async fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, &block_pos.down()).await
    }
}
fn supports_fungus(block: &Block) -> bool {
    block.has_tag(&tag::Block::MINECRAFT_DIRT)
        || block == &Block::FARMLAND
        || block == &Block::WARPED_NYLIUM
        || block == &Block::CRIMSON_NYLIUM
        || block == &Block::SOUL_SOIL
        || block == &Block::MUD
        || block == &Block::MUDDY_MANGROVE_ROOTS
}

use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs};
use crate::block::{GetStateForNeighborUpdateArgs, blocks::plant::PlantBlockBase};
use pumpkin_data::Block;
use pumpkin_world::BlockStateId;
pub struct Fungus;

impl BlockMetadata for Fungus {
    fn ids() -> Box<[u16]> {
        [Block::CRIMSON_FUNGUS.id, Block::WARPED_FUNGUS.id].into()
    }
}

impl BlockBehaviour for Fungus {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let block = args.block_accessor.get_block(&args.position.down()).await;
            if supports_fungus(block) {
                return true;
            }
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
    }
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let block = args.world.get_block(&args.position.down()).await;
            if supports_fungus(block) {
                return args.state_id;
            }
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
impl PlantBlockBase for Fungus {}
//other supporting block types are included in PlantBlockBase
fn supports_fungus(block: &Block) -> bool {
    block == &Block::WARPED_NYLIUM || block == &Block::CRIMSON_NYLIUM || block == &Block::SOUL_SOIL
}

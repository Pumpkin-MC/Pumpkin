use crate::block::blocks::plant::fungus::supports_fungus;
use crate::block::{BlockBehaviour, BlockFuture, CanPlaceAtArgs};
use crate::block::{GetStateForNeighborUpdateArgs, blocks::plant::PlantBlockBase};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;
#[pumpkin_block("minecraft:nether_sprouts")]
pub struct NetherSproutsBlock;

impl BlockBehaviour for NetherSproutsBlock {
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
impl PlantBlockBase for NetherSproutsBlock {
    async fn can_plant_on_top(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        let block = block_accessor.get_block(pos).await;
        // Nether Sprouts share the same soil requirements as Fungi
        supports_fungus(block)
    }
}

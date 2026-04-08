use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, BrokenArgs, CanPlaceAtArgs,
    GetStateForNeighborUpdateArgs, PlacedArgs,
};
use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

pub struct WeepingVinesBlock;
impl BlockMetadata for WeepingVinesBlock {
    fn ids() -> Box<[u16]> {
        [Block::WEEPING_VINES.id, Block::WEEPING_VINES_PLANT.id].into()
    }
}

impl BlockBehaviour for WeepingVinesBlock {
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
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let support_pos = args.position.up();
            let support_block = args.world.get_block(&support_pos).await;
            if support_block == &Block::WEEPING_VINES {
                args.world
                    .set_block_state(
                        &support_pos,
                        Block::WEEPING_VINES_PLANT.default_state.id,
                        BlockFlags::empty(),
                    )
                    .await;
            }
        })
    }
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let support_pos = args.position.up();
            let support_block = args.world.get_block(&support_pos).await;
            if support_block == &Block::WEEPING_VINES_PLANT {
                args.world
                    .set_block_state(
                        &support_pos,
                        Block::WEEPING_VINES.default_state.id,
                        BlockFlags::empty(),
                    )
                    .await;
            }
        })
    }
}

impl PlantBlockBase for WeepingVinesBlock {
    async fn can_place_at(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        // Determine support block
        let support_pos = pos.up();
        let (support_block, support_block_state) =
            block_accessor.get_block_and_state(&support_pos).await;

        if support_block == &Block::WEEPING_VINES || support_block == &Block::WEEPING_VINES_PLANT {
            return true;
        }
        if support_block_state.is_side_solid(pumpkin_data::BlockDirection::Down)
            && support_block.is_solid()
        {
            return true;
        }
        false
    }
    async fn get_state_for_neighbor_update(
        &self,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        block_state: BlockStateId,
    ) -> BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos).await {
            return Block::AIR.default_state.id;
        }
        block_state
    }
}

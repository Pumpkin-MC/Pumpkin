use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, BrokenArgs, CanPlaceAtArgs,
    GetStateForNeighborUpdateArgs, PlacedArgs,
};
use pumpkin_data::Block;
use pumpkin_data::block_properties::{BlockProperties, WaterLikeProperties};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
pub struct KelpBlock;

impl BlockMetadata for KelpBlock {
    fn ids() -> Box<[u16]> {
        [Block::KELP.id, Block::KELP_PLANT.id].into()
    }
}

impl BlockBehaviour for KelpBlock {
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
            let support_pos = args.position.down();
            let support_block = args.world.get_block(&support_pos).await;
            if support_block == &Block::KELP {
                args.world
                    .set_block_state(
                        &support_pos,
                        Block::KELP_PLANT.default_state.id,
                        BlockFlags::empty(),
                    )
                    .await;
            }
        })
    }
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let support_pos = args.position.down();
            let support_block = args.world.get_block(&support_pos).await;
            if support_block == &Block::KELP_PLANT {
                args.world
                    .set_block_state(
                        &support_pos,
                        Block::KELP.default_state.id,
                        BlockFlags::empty(),
                    )
                    .await;
                args.world
                    .set_block_state(
                        args.position,
                        Block::WATER.default_state.id,
                        BlockFlags::empty(),
                    )
                    .await;
            }
        })
    }
}

impl PlantBlockBase for KelpBlock {
    async fn can_plant_on_top(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        // Determine support block
        let support_pos = pos;
        let (replacing_block, replacing_block_state) =
            block_accessor.get_block_and_state(&pos.up()).await;
        let support_block = block_accessor.get_block(support_pos).await;
        if replacing_block == &Block::WATER {
            let water_props =
                WaterLikeProperties::from_state_id(replacing_block_state.id, replacing_block);

            //Only allow placing kelp on either full water or downward flowing water
            if water_props.level as u8 != 0 && water_props.level as u8 != 8 {
                return false;
            }
        } else {
            //Replacing block can also be a kelp_plant or kelp in case this is an neighbour update check
            if replacing_block != &Block::KELP_PLANT && replacing_block != &Block::KELP {
                return false;
            }
        }
        // If placing the base kelp block, allow placement on water or on other kelp segments.
        if support_block == &Block::KELP || support_block == &Block::KELP_PLANT {
            return true;
        }
        if support_block.default_state.is_full_cube() {
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
            return Block::WATER.default_state.id;
        }
        block_state
    }
}

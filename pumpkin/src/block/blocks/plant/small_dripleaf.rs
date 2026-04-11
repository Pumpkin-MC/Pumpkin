use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    PlacedArgs,
};
use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    BlockProperties, DoubleBlockHalf, SmallDripleafLikeProperties,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

#[pumpkin_block("minecraft:small_dripleaf")]
pub struct SmallDripleafBlock;

impl BlockBehaviour for SmallDripleafBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
    }
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let facing = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();
            let mut small_dripleaf_props = SmallDripleafLikeProperties::default(args.block);

            small_dripleaf_props.facing = facing;
            small_dripleaf_props.waterlogged = args.replacing.water_source();
            small_dripleaf_props.half = DoubleBlockHalf::Lower;

            small_dripleaf_props.to_state_id(args.block)
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
            let lower_small_dripleaf_props =
                SmallDripleafLikeProperties::from_state_id(args.state_id, args.block);
            if lower_small_dripleaf_props.half != DoubleBlockHalf::Lower {
                return;
            }

            let mut upper_small_dripleaf_props =
                SmallDripleafLikeProperties::default(&Block::SMALL_DRIPLEAF);

            let upper_block = args.world.get_block(&args.position.up()).await;
            upper_small_dripleaf_props.facing = lower_small_dripleaf_props.facing;
            upper_small_dripleaf_props.waterlogged = upper_block == &Block::WATER;
            upper_small_dripleaf_props.half = DoubleBlockHalf::Upper;

            args.world
                .set_block_state(
                    &args.position.up(),
                    upper_small_dripleaf_props.to_state_id(&Block::SMALL_DRIPLEAF),
                    BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
                )
                .await;
        })
    }
}
fn is_small_dripleaf_waterlogged(state_id: BlockStateId) -> bool {
    let dripleaf_props =
        SmallDripleafLikeProperties::from_state_id(state_id, &Block::SMALL_DRIPLEAF);
    dripleaf_props.waterlogged
}
impl PlantBlockBase for SmallDripleafBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let support_block = block_accessor.get_block(pos).await;

        if support_block == &Block::SMALL_DRIPLEAF {
            return true;
        }
        let upper_block = block_accessor.get_block(&pos.up_height(2)).await;
        if upper_block != &Block::AIR
            && upper_block != &Block::WATER
            && upper_block != &Block::SMALL_DRIPLEAF
        {
            return false;
        }
        let (replacing_block, replacing_block_state) =
            block_accessor.get_block_and_state(&pos.up()).await;
        if replacing_block == &Block::SMALL_DRIPLEAF && replacing_block_state.is_waterlogged() {
            //in case of neighbor update check
            supports_small_dripleaf(support_block, true)
        } else {
            supports_small_dripleaf(support_block, replacing_block == &Block::WATER)
        }
    }

    async fn get_state_for_neighbor_update(
        &self,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        block_state: BlockStateId,
    ) -> BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos).await {
            if is_small_dripleaf_waterlogged(block_state) {
                return Block::WATER.default_state.id;
            }
            return Block::AIR.default_state.id;
        }
        let upper_block = block_accessor.get_block(&block_pos.up()).await;
        let below_blow = block_accessor.get_block(&block_pos.down()).await;
        if upper_block != &Block::SMALL_DRIPLEAF && below_blow != &Block::SMALL_DRIPLEAF {
            if is_small_dripleaf_waterlogged(block_state) {
                return Block::WATER.default_state.id;
            }
            return Block::AIR.default_state.id;
        }
        block_state
    }
}
fn supports_small_dripleaf(support_block: &Block, underwater: bool) -> bool {
    if support_block == &Block::CLAY || support_block == &Block::MOSS_BLOCK {
        return true;
    }
    underwater
        && (support_block == &Block::COARSE_DIRT
            || support_block == &Block::DIRT
            || support_block == &Block::FARMLAND
            || support_block == &Block::GRASS_BLOCK
            || support_block == &Block::MUD
            || support_block == &Block::MYCELIUM
            || support_block == &Block::PODZOL
            || support_block == &Block::ROOTED_DIRT)
}

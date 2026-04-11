use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, BrokenArgs, CanPlaceAtArgs,
    GetStateForNeighborUpdateArgs, OnPlaceArgs, PlacedArgs,
};
use pumpkin_data::block_properties::{
    BigDripleafLikeProperties, BlockProperties, HorizontalFacing, LadderLikeProperties,
};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
pub struct BigDripleafBlock;

impl BlockMetadata for BigDripleafBlock {
    fn ids() -> Box<[u16]> {
        [Block::BIG_DRIPLEAF_STEM.id, Block::BIG_DRIPLEAF.id].into()
    }
}
impl BlockBehaviour for BigDripleafBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
    }
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let (support_block, support_block_state_id) = args
                .world
                .get_block_and_state_id(&args.position.down())
                .await;
            let facing = if support_block == &Block::BIG_DRIPLEAF_STEM
                || support_block == &Block::BIG_DRIPLEAF
            {
                get_dripleaf_facing_dir(support_block, support_block_state_id)
            } else {
                args.player
                    .living_entity
                    .entity
                    .get_horizontal_facing()
                    .opposite()
            };
            if args.block == &Block::BIG_DRIPLEAF {
                let mut dripleaf_props = BigDripleafLikeProperties::default(args.block);

                dripleaf_props.facing = facing;
                dripleaf_props.waterlogged = args.replacing.water_source();

                return dripleaf_props.to_state_id(args.block);
            }

            let mut dripleaf_stem_props = LadderLikeProperties::default(args.block);

            dripleaf_stem_props.facing = facing;
            dripleaf_stem_props.waterlogged = args.replacing.water_source();

            dripleaf_stem_props.to_state_id(args.block)
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

    /// if leaf is placed on top of another leaf, turn the lower one into a stem.
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let support_pos = args.position.down();
            let (support_block, support_state_id) =
                args.world.get_block_and_state_id(&support_pos).await;
            if support_block == &Block::BIG_DRIPLEAF {
                let old_facing = get_dripleaf_facing_dir(support_block, support_state_id);
                let old_waterlogged = is_dripleaf_waterlogged(support_block, support_state_id);
                let mut dripleaf_stem_props =
                    LadderLikeProperties::default(&Block::BIG_DRIPLEAF_STEM);

                dripleaf_stem_props.facing = old_facing;
                dripleaf_stem_props.waterlogged = old_waterlogged;
                args.world
                    .set_block_state(
                        &support_pos,
                        dripleaf_stem_props.to_state_id(&Block::BIG_DRIPLEAF_STEM),
                        BlockFlags::empty(),
                    )
                    .await;
            }
        })
    }

    /// if the leaf is broken, turn the stem below into a leaf.
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let support_pos = args.position.down();
            let (support_block, support_state_id) =
                args.world.get_block_and_state_id(&support_pos).await;
            if support_block == &Block::BIG_DRIPLEAF_STEM {
                let old_facing = get_dripleaf_facing_dir(args.block, args.state.id);
                let old_waterlogged = is_dripleaf_waterlogged(support_block, support_state_id);

                let mut dripleaf_props = BigDripleafLikeProperties::default(&Block::BIG_DRIPLEAF);
                dripleaf_props.facing = old_facing;
                dripleaf_props.waterlogged = old_waterlogged;
                args.world
                    .set_block_state(
                        &support_pos,
                        dripleaf_props.to_state_id(&Block::BIG_DRIPLEAF),
                        BlockFlags::empty(),
                    )
                    .await;
            }
        })
    }
}
fn get_dripleaf_facing_dir(block: &Block, state_id: BlockStateId) -> HorizontalFacing {
    if block == &Block::BIG_DRIPLEAF {
        let dripleaf_props =
            BigDripleafLikeProperties::from_state_id(state_id, &Block::BIG_DRIPLEAF);
        return dripleaf_props.facing;
    }
    let dripleaf_stem_props = LadderLikeProperties::from_state_id(state_id, block);
    dripleaf_stem_props.facing
}

fn is_dripleaf_waterlogged(block: &Block, state_id: BlockStateId) -> bool {
    if block == &Block::BIG_DRIPLEAF {
        let dripleaf_props =
            BigDripleafLikeProperties::from_state_id(state_id, &Block::BIG_DRIPLEAF);
        return dripleaf_props.waterlogged;
    }
    let dripleaf_stem_props = LadderLikeProperties::from_state_id(state_id, block);
    dripleaf_stem_props.waterlogged
}
impl PlantBlockBase for BigDripleafBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let support_block = block_accessor.get_block(pos).await;

        if support_block == &Block::BIG_DRIPLEAF || support_block == &Block::BIG_DRIPLEAF_STEM {
            return true;
        }

        support_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_BIG_DRIPLEAF)
    }

    async fn get_state_for_neighbor_update(
        &self,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        block_state: BlockStateId,
    ) -> BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos).await {
            let block = block_accessor.get_block(block_pos).await;
            if is_dripleaf_waterlogged(block, block_state) {
                return Block::WATER.default_state.id;
            }
            return Block::AIR.default_state.id;
        }
        block_state
    }
}

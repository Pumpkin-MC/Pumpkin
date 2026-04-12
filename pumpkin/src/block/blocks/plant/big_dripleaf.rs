use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::big_dripleaf_stem::{
    BigDripleafStemLikeProperties, handle_big_dripleaf_breaking,
};
use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnPlaceArgs, PlacedArgs,
};
use pumpkin_data::block_properties::{
    BigDripleafLikeProperties, BlockProperties, HorizontalFacing,
};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

#[pumpkin_block("minecraft:big_dripleaf")]
pub struct BigDripleafBlock;

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
            let facing = if support_block == &Block::BIG_DRIPLEAF {
                get_dripleaf_facing_dir(support_block_state_id)
            } else {
                args.player
                    .living_entity
                    .entity
                    .get_horizontal_facing()
                    .opposite()
            };
            let mut dripleaf_props = BigDripleafLikeProperties::default(args.block);

            dripleaf_props.facing = facing;
            dripleaf_props.waterlogged = args.replacing.water_source();

            dripleaf_props.to_state_id(args.block)
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
                let old_dripleaf_props = BigDripleafLikeProperties::from_state_id(
                    support_state_id,
                    &Block::BIG_DRIPLEAF,
                );
                let mut dripleaf_stem_props =
                    BigDripleafStemLikeProperties::default(&Block::BIG_DRIPLEAF_STEM);

                dripleaf_stem_props.facing = old_dripleaf_props.facing;
                dripleaf_stem_props.waterlogged = old_dripleaf_props.waterlogged;
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
        Box::pin(async move { handle_big_dripleaf_breaking(args.world, args.position).await })
    }
}
fn get_dripleaf_facing_dir(state_id: BlockStateId) -> HorizontalFacing {
    let dripleaf_props = BigDripleafLikeProperties::from_state_id(state_id, &Block::BIG_DRIPLEAF);
    dripleaf_props.facing
}

fn is_dripleaf_waterlogged(state_id: BlockStateId) -> bool {
    let dripleaf_props = BigDripleafLikeProperties::from_state_id(state_id, &Block::BIG_DRIPLEAF);
    dripleaf_props.waterlogged
}
impl PlantBlockBase for BigDripleafBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let support_block = block_accessor.get_block(pos).await;
        can_plant_dripleaf_on_top(support_block)
    }

    async fn get_state_for_neighbor_update(
        &self,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        block_state: BlockStateId,
    ) -> BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos).await {
            if is_dripleaf_waterlogged(block_state) {
                return Block::WATER.default_state.id;
            }
            return Block::AIR.default_state.id;
        }
        block_state
    }
}
#[must_use]
pub fn can_plant_dripleaf_on_top(support_block: &Block) -> bool {
    if support_block == &Block::BIG_DRIPLEAF || support_block == &Block::BIG_DRIPLEAF_STEM {
        return true;
    }

    support_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_BIG_DRIPLEAF)
}

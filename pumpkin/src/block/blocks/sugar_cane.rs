use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, CactusLikeProperties, EnumVariants, Integer0To15},
    tag,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnScheduledTickArgs, RandomTickArgs,
};

#[pumpkin_block("minecraft:sugar_cane")]
pub struct SugarCaneBlock;

impl BlockBehaviour for SugarCaneBlock {
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_place_at(args.world.as_ref(), args.position).await {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if args
                .world
                .get_block_state(&args.position.up())
                .await
                .is_air()
            {
                let state_id = args.world.get_block_state(args.position).await.id;
                let age = CactusLikeProperties::from_state_id(state_id, args.block).age;
                if age == Integer0To15::L15 {
                    args.world
                        .set_block_state(&args.position.up(), state_id, BlockFlags::empty())
                        .await;
                    let props = CactusLikeProperties {
                        age: Integer0To15::L0,
                    };
                    args.world
                        .set_block_state(
                            args.position,
                            props.to_state_id(args.block),
                            BlockFlags::empty(),
                        )
                        .await;
                } else {
                    let props = CactusLikeProperties {
                        age: Integer0To15::from_index(age.to_index() + 1),
                    };
                    args.world
                        .set_block_state(
                            args.position,
                            props.to_state_id(args.block),
                            BlockFlags::empty(),
                        )
                        .await;
                }
            }
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position).await {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
            }
            args.state_id
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move { can_place_at(args.block_accessor, args.position).await })
    }
}

async fn can_place_at(block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    let block_below = block_accessor.get_block(&block_pos.down()).await;

    if block_below == &Block::SUGAR_CANE {
        return true;
    }

    if block_below.has_tag(&tag::Block::MINECRAFT_DIRT)
        || block_below.has_tag(&tag::Block::MINECRAFT_SAND)
    {
        for direction in HorizontalFacing::all() {
            let block = block_accessor
                .get_block(&block_pos.down().offset(direction.to_offset()))
                .await;

            if block == &Block::WATER || block == &Block::FROSTED_ICE {
                return true;
            }
        }
    }

    false
}

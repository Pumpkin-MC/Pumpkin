use crate::block::{
    BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs,
};
use crate::entity::EntityBase;
use pumpkin_data::block_properties::{BlockProperties, WhiteBannerLikeProperties};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

#[pumpkin_block_from_tag("minecraft:banners")]
pub struct BannerBlock;

impl BlockBehaviour for BannerBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = WhiteBannerLikeProperties::default(args.block);
            props.rotation = args.player.get_entity().get_flipped_rotation_16();
            props.to_state_id(args.block)
        })
    }

    fn can_place_at<'a>(&'a self, args: crate::block::CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move { can_place_at(args.block_accessor, args.position).await })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_place_at(args.world.as_ref(), args.position).await {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
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
}

async fn can_place_at(world: &dyn BlockAccessor, position: &BlockPos) -> bool {
    let state = world.get_block_state(&position.down()).await;
    state.is_solid()
}

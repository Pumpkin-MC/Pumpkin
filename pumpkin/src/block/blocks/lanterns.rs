use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnPlaceArgs, OnScheduledTickArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

pub struct LanternBlock;

impl BlockMetadata for LanternBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::LANTERN.name, Block::SOUL_LANTERN.name]
    }
}

impl BlockBehaviour for LanternBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props =
                pumpkin_data::block_properties::LanternLikeProperties::default(args.block);
            props.r#waterlogged = args.replacing.water_source();

            let block_up_state = args.world.get_block_state(&args.position.up()).await;
            if block_up_state.is_center_solid(BlockDirection::Down) {
                props.r#hanging = true;
            }

            props.to_state_id(args.block)
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move { can_place_at(args.world.unwrap(), args.position).await })
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

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position).await {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }
}

async fn can_place_at(world: &World, position: &BlockPos) -> bool {
    //idk why this don't update with .is_center_solid so this is a 'temporary patch'
    if world
        .get_block(&position.down())
        .await
        .has_tag(&tag::Block::C_FENCE_GATES)
    {
        let fence_gate_props =
            pumpkin_data::block_properties::OakFenceGateLikeProperties::from_state_id(
                world.get_block_state_id(&position.down()).await,
                world.get_block(&position.down()).await,
            );

        if fence_gate_props.open {
            return false;
        }
    }
    let (block_down, block_down_state) = world.get_block_and_state(&position.down()).await;
    let block_up_state = world.get_block_state(&position.up()).await;
    block_down_state.is_center_solid(BlockDirection::Up)
        || block_up_state.is_center_solid(BlockDirection::Down)
        || block_down.has_tag(&tag::Block::MINECRAFT_UNSTABLE_BOTTOM_CENTER)
}

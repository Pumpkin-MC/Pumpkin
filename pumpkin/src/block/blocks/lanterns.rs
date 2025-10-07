use crate::block::{
    BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs,
};
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{BlockDirection, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_data::tag::get_tag_values;
use pumpkin_data::tag::RegistryKey;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block_from_tag("minecraft:chains")]
pub struct LanternBlock;

#[async_trait]
impl BlockBehaviour for LanternBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = pumpkin_data::block_properties::LanternLikeProperties::default(args.block);
        props.r#waterlogged = args.replacing.water_source();

        let block_up_state = args.world.get_block_state(&args.position.up()).await;
        if block_up_state.is_center_solid(BlockDirection::Down) {
            props.r#hanging = true;
        }

        props.to_state_id(args.block)
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.world.unwrap(), args.position).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !can_place_at(args.world, args.position).await {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                .await;
        }
        args.state_id
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !can_place_at(args.world, args.position).await {
            args.world
                .break_block(args.position, None, BlockFlags::empty())
                .await;
        }
    }
}

async fn can_place_at(world: &World, position: &BlockPos) -> bool {
    //idk why this don't update with .is_center_solid so this is a 'temporary patch'
    if world
        .get_block(&position.down())
        .await
        .is_tagged_with_by_tag(&tag::Block::C_FENCE_GATES)
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
        || block_down.is_tagged_with_by_tag(&tag::Block::MINECRAFT_UNSTABLE_BOTTOM_CENTER)
}

use crate::block::pumpkin_block::{
    BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs,
    PumpkinBlock,
};
use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

pub struct LanternBlock;

impl BlockMetadata for LanternBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::LANTERN.name, Block::SOUL_LANTERN.name]
    }
}

#[async_trait]
impl PumpkinBlock for LanternBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = pumpkin_data::block_properties::LanternLikeProperties::default(args.block);
        props.r#waterlogged = args.replacing.water_source();
        if args
            .world
            .get_block_state(&args.position.down())
            .await
            .is_air()
        {
            props.r#hanging = true;
        }
        props.to_state_id(args.block)
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        args.block_accessor
            .get_block_state(&args.position.down())
            .await
            .is_center_solid(BlockDirection::Up)
            || args
                .block_accessor
                .get_block_state(&args.position.up())
                .await
                .is_center_solid(BlockDirection::Down)
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
        if !can_place_at(args.world.as_ref(), args.position).await {
            args.world
                .break_block(args.position, None, BlockFlags::empty())
                .await;
        }
    }
}

async fn can_place_at(block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    !block_accessor
        .get_block_state(&block_pos.down())
        .await
        .is_air()
}

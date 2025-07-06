use crate::block::pumpkin_block::{
    BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnScheduledTickArgs, PumpkinBlock,
};
use async_trait::async_trait;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

pub struct CarpetBlock;

impl BlockMetadata for CarpetBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:wool_carpets").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for CarpetBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.location).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !can_place_at(args.world, args.location).await {
            args.world
                .schedule_block_tick(args.block, *args.location, 1, TickPriority::Normal)
                .await;
        }
        args.state_id
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !can_place_at(args.world.as_ref(), args.location).await {
            args.world
                .break_block(args.location, None, BlockFlags::empty())
                .await;
        }
    }
}

#[pumpkin_block("minecraft:moss_carpet")]
pub struct MossCarpetBlock;

#[async_trait]
impl PumpkinBlock for MossCarpetBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.location).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !can_place_at(args.world, args.location).await {
            args.world
                .schedule_block_tick(args.block, *args.location, 1, TickPriority::Normal)
                .await;
        }
        args.state_id
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !can_place_at(args.world.as_ref(), args.location).await {
            args.world
                .break_block(args.location, None, BlockFlags::empty())
                .await;
        }
    }
}

#[pumpkin_block("minecraft:pale_moss_carpet")]
pub struct PaleMossCarpetBlock;

#[async_trait]
impl PumpkinBlock for PaleMossCarpetBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.location).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !can_place_at(args.world, args.location).await {
            args.world
                .schedule_block_tick(args.block, *args.location, 1, TickPriority::Normal)
                .await;
        }
        args.state_id
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !can_place_at(args.world.as_ref(), args.location).await {
            args.world
                .break_block(args.location, None, BlockFlags::empty())
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

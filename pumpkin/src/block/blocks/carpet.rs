//! Vanilla `CarpetBlock`: updateShape returns AIR immediately if !canSurvive
//! (floor not empty). No scheduled/random tick for attachment.

use crate::block::blocks::support::{air_if_unsupported, has_floor_support};
use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnNeighborUpdateArgs,
};
use pumpkin_data::BlockStateId;
use pumpkin_macros::{pumpkin_block, pumpkin_block_from_tag};
use pumpkin_world::world::BlockFlags;

#[pumpkin_block_from_tag("minecraft:wool_carpets")]
pub struct CarpetBlock;

impl BlockBehaviour for CarpetBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        has_floor_support(args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            air_if_unsupported(has_floor_support(args.world, args.position), args.state_id)
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !has_floor_support(args.world.as_ref(), args.position) {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        })
    }
}

#[pumpkin_block("minecraft:moss_carpet")]
pub struct MossCarpetBlock;

impl BlockBehaviour for MossCarpetBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        has_floor_support(args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            air_if_unsupported(has_floor_support(args.world, args.position), args.state_id)
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !has_floor_support(args.world.as_ref(), args.position) {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        })
    }
}

#[pumpkin_block("minecraft:pale_moss_carpet")]
pub struct PaleMossCarpetBlock;

impl BlockBehaviour for PaleMossCarpetBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        has_floor_support(args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            air_if_unsupported(has_floor_support(args.world, args.position), args.state_id)
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !has_floor_support(args.world.as_ref(), args.position) {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        })
    }
}

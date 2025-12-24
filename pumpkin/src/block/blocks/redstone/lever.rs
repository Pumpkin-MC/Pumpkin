use std::sync::Arc;

use crate::block::{
    BlockFuture, CanPlaceAtArgs, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
    GetStateForNeighborUpdateArgs, OnPlaceArgs, OnStateReplacedArgs,
    blocks::abstract_wall_mounting::WallMountedBlock,
};
use pumpkin_data::{
    Block, BlockDirection, HorizontalFacingExt,
    block_properties::{BlockFace, BlockProperties, LeverLikeProperties},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockFlags};

use crate::{
    block::{
        registry::BlockActionResult,
        {BlockBehaviour, NormalUseArgs},
    },
    world::World,
};

async fn toggle_lever(world: &Arc<World>, block_pos: &BlockPos) {
    let (block, state) = world.get_block_and_state_id(block_pos).await;

    let mut lever_props = LeverLikeProperties::from_state_id(state, block);
    lever_props.powered = !lever_props.powered;
    world
        .set_block_state(
            block_pos,
            lever_props.to_state_id(block),
            BlockFlags::NOTIFY_ALL,
        )
        .await;

    LeverBlock::update_neighbors(world, block_pos, &lever_props).await;
}

#[pumpkin_block("minecraft:lever")]
pub struct LeverBlock;

impl BlockBehaviour for LeverBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            toggle_lever(args.world, args.position).await;

            BlockActionResult::Success
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let lever_props = LeverLikeProperties::from_state_id(args.state.id, args.block);
            if lever_props.powered { 15 } else { 0 }
        })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let lever_props = LeverLikeProperties::from_state_id(args.state.id, args.block);
            if lever_props.powered && lever_props.get_direction() == args.direction {
                15
            } else {
                0
            }
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !args.moved {
                let lever_props = LeverLikeProperties::from_state_id(args.old_state_id, args.block);
                if lever_props.powered {
                    Self::update_neighbors(args.world, args.position, &lever_props).await;
                }
            }
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props =
                LeverLikeProperties::from_state_id(args.block.default_state.id, args.block);
            (props.face, props.facing) =
                WallMountedBlock::get_placement_face(self, args.player, args.direction);

            props.to_state_id(args.block)
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            WallMountedBlock::can_place_at(self, args.block_accessor, args.position, args.direction)
                .await
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move { WallMountedBlock::get_state_for_neighbor_update(self, args).await })
    }
}

impl WallMountedBlock for LeverBlock {
    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection {
        let props = LeverLikeProperties::from_state_id(state_id, block);
        match props.face {
            BlockFace::Floor => BlockDirection::Up,
            BlockFace::Ceiling => BlockDirection::Down,
            BlockFace::Wall => props.facing.to_block_direction(),
        }
    }
}

impl LeverBlock {
    async fn update_neighbors(
        world: &Arc<World>,
        block_pos: &BlockPos,
        lever_props: &LeverLikeProperties,
    ) {
        let direction = lever_props.get_direction().opposite();
        world.update_neighbors(block_pos, None).await;
        world
            .update_neighbors(&block_pos.offset(direction.to_offset()), None)
            .await;
    }
}

pub trait LeverLikePropertiesExt {
    fn get_direction(&self) -> BlockDirection;
}

impl LeverLikePropertiesExt for LeverLikeProperties {
    fn get_direction(&self) -> BlockDirection {
        match self.face {
            BlockFace::Ceiling => BlockDirection::Down,
            BlockFace::Floor => BlockDirection::Up,
            BlockFace::Wall => self.facing.to_block_direction(),
        }
    }
}

use std::sync::Arc;

use crate::{
    block::{BlockIsReplacing, blocks::abstruct_wall_mounting::WallMountedBlock},
    entity::player::Player,
};
use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection, BlockState, HorizontalFacingExt,
    block_properties::{BlockFace, BlockProperties, LeverLikeProperties},
    item::Item,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};

use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    server::Server,
    world::World,
};

async fn toggle_lever(world: &Arc<World>, block_pos: &BlockPos) {
    let (block, state) = world.get_block_and_block_state(block_pos).await;

    let mut lever_props = LeverLikeProperties::from_state_id(state.id, &block);
    lever_props.powered = !lever_props.powered;
    world
        .set_block_state(
            block_pos,
            lever_props.to_state_id(&block),
            BlockFlags::NOTIFY_ALL,
        )
        .await;

    LeverBlock::update_neighbors(world, block_pos, &lever_props).await;
}

#[pumpkin_block("minecraft:lever")]
pub struct LeverBlock;

#[async_trait]
impl PumpkinBlock for LeverBlock {
    async fn use_with_item(
        &self,
        _block: &Block,
        _player: &Player,
        location: BlockPos,
        _item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        toggle_lever(world, &location).await;
        BlockActionResult::Consume
    }

    async fn normal_use(
        &self,
        _block: &Block,
        _player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        toggle_lever(world, &location).await;
    }

    async fn emits_redstone_power(
        &self,
        _block: &Block,
        _state: &BlockState,
        _direction: BlockDirection,
    ) -> bool {
        true
    }

    async fn get_weak_redstone_power(
        &self,
        block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        state: &BlockState,
        _direction: BlockDirection,
    ) -> u8 {
        let lever_props = LeverLikeProperties::from_state_id(state.id, block);
        if lever_props.powered { 15 } else { 0 }
    }

    async fn get_strong_redstone_power(
        &self,
        block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        let lever_props = LeverLikeProperties::from_state_id(state.id, block);
        if lever_props.powered && lever_props.get_direction() == direction {
            15
        } else {
            0
        }
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        block: &Block,
        location: BlockPos,
        old_state_id: BlockStateId,
        moved: bool,
    ) {
        if !moved {
            let lever_props = LeverLikeProperties::from_state_id(old_state_id, block);
            if lever_props.powered {
                Self::update_neighbors(world, &location, &lever_props).await;
            }
        }
    }

    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        direction: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let mut props = LeverLikeProperties::from_state_id(block.default_state.id, block);
        (props.face, props.facing) = WallMountedBlock::get_placement_face(self, player, direction);

        props.to_state_id(block)
    }

    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        pos: &BlockPos,
        face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        WallMountedBlock::can_place_at(self, block_accessor, pos, face).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state: BlockStateId,
        pos: &BlockPos,
        direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        WallMountedBlock::get_state_for_neighbor_update(self, state, block, direction, world, pos)
            .await
            .unwrap_or(state)
    }
}

#[async_trait]
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

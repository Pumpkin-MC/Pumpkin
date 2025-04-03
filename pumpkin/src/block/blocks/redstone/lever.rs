use std::sync::Arc;

use crate::entity::player::Player;
use crate::world::BlockFlags;
use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockFace, BlockState, LeverLikeProperties};
use pumpkin_data::{
    block::{BlockProperties, HorizontalFacing},
    item::Item,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{BlockDirection, HorizontalFacingExt};

use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    server::Server,
    world::World,
};

async fn toggle_lever(world: &Arc<World>, block_pos: &BlockPos) {
    let (block, state) = world.get_block_and_block_state(block_pos).await.unwrap();

    let mut lever_props = LeverLikeProperties::from_state_id(state.id, &block);
    lever_props.powered = lever_props.powered.flip();
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
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        block: &Block,
        face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        player_direction: &HorizontalFacing,
        _other: bool,
    ) -> u16 {
        let mut lever_props = LeverLikeProperties::from_state_id(block.default_state_id, block);

        match face {
            BlockDirection::Up => lever_props.face = BlockFace::Ceiling,
            BlockDirection::Down => lever_props.face = BlockFace::Floor,
            _ => lever_props.face = BlockFace::Wall,
        }

        if face == &BlockDirection::Up || face == &BlockDirection::Down {
            lever_props.facing = *player_direction;
        } else {
            lever_props.facing = face.opposite().to_cardinal_direction();
        }

        lever_props.to_state_id(block)
    }

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
        _direction: &BlockDirection,
    ) -> bool {
        true
    }

    async fn get_weak_redstone_power(
        &self,
        block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        state: &BlockState,
        _direction: &BlockDirection,
    ) -> u8 {
        let lever_props = LeverLikeProperties::from_state_id(state.id, block);
        if lever_props.powered.to_bool() { 15 } else { 0 }
    }

    async fn get_strong_redstone_power(
        &self,
        block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        state: &BlockState,
        direction: &BlockDirection,
    ) -> u8 {
        let lever_props = LeverLikeProperties::from_state_id(state.id, block);
        if lever_props.powered.to_bool() && &lever_props.get_direction() == direction {
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
        old_state_id: u16,
        moved: bool,
    ) {
        if !moved {
            let lever_props = LeverLikeProperties::from_state_id(old_state_id, block);
            if lever_props.powered.to_bool() {
                Self::update_neighbors(world, &location, &lever_props).await;
            }
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

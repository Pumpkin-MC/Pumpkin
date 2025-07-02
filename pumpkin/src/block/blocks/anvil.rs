use crate::block::{BlockIsReplacing, pumpkin_block::PumpkinBlock};
use crate::server::Server;
use crate::{entity::player::Player, world::World};
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, WallTorchLikeProperties};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

#[pumpkin_block("minecraft:anvil")]
pub struct AnvilBlock;

#[async_trait]
impl PumpkinBlock for AnvilBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        on_place(player, block)
    }
}

#[pumpkin_block("minecraft:chipped_anvil")]
pub struct ChippedAnvilBlock;

#[async_trait]
impl PumpkinBlock for ChippedAnvilBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        on_place(player, block)
    }
}

#[pumpkin_block("minecraft:damaged_anvil")]
pub struct DamagedAnvilBlock;

#[async_trait]
impl PumpkinBlock for DamagedAnvilBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        on_place(player, block)
    }
}

fn on_place(player: &Player, block: &Block) -> BlockStateId {
    let dir = player
        .living_entity
        .entity
        .get_horizontal_facing()
        .rotate_clockwise();

    let mut props = WallTorchLikeProperties::default(block);

    props.facing = dir;
    props.to_state_id(block)
}

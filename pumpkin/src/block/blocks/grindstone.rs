use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection, HorizontalFacingExt,
    block_properties::{BlockFace, BlockProperties, GrindstoneLikeProperties},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

use crate::server::Server;
use crate::world::World;
use crate::{
    block::{BlockIsReplacing, pumpkin_block::PumpkinBlock},
    entity::player::Player,
};

use super::abstruct_wall_mounting::WallMountedBlock;

#[pumpkin_block("minecraft:grindstone")]
pub struct GrindstoneBlock;

#[async_trait]
impl PumpkinBlock for GrindstoneBlock {
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
        let mut props = GrindstoneLikeProperties::from_state_id(block.default_state.id, block);
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
impl WallMountedBlock for GrindstoneBlock {
    async fn can_place_at(
        &self,
        _world: &dyn BlockAccessor,
        _pos: &BlockPos,
        _direction: BlockDirection,
    ) -> bool {
        true
    }

    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection {
        let props = GrindstoneLikeProperties::from_state_id(state_id, block);
        match props.face {
            BlockFace::Floor => BlockDirection::Up,
            BlockFace::Ceiling => BlockDirection::Down,
            BlockFace::Wall => props.facing.to_block_direction(),
        }
    }
}

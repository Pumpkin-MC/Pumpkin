use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::EntityBase;
use crate::world::World;
use crate::world::portal::nether::NetherPortal;
use async_trait::async_trait;
use pumpkin_data::block_properties::{Axis, BlockProperties, NetherPortalLikeProperties};
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

#[pumpkin_block("minecraft:nether_portal")]
pub struct NetherPortalBlock;

#[async_trait]
impl PumpkinBlock for NetherPortalBlock {
    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        _block: &Block,
        state: BlockStateId,
        pos: &BlockPos,
        direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        neighbor_state: BlockStateId,
    ) -> BlockStateId {
        let axis = direction.to_axis();
        let is_horizontal = axis == Axis::X && axis == Axis::Z;
        let state_axis =
            NetherPortalLikeProperties::from_state_id(state, &Block::NETHER_PORTAL).axis;
        if is_horizontal
            || neighbor_state == state
            || NetherPortal::get_on_axis(world, pos, state_axis)
                .await
                .is_some_and(|e| e.was_already_valid())
        {
            return state;
        }
        Block::AIR.default_state_id
    }

    async fn on_entity_collision(
        &self,
        _world: &Arc<World>,
        _entity: &dyn EntityBase,
        _pos: BlockPos,
        _block: Block,
        _state: BlockState,
    ) {
    }
}

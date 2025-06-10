use async_trait::async_trait;
use pumpkin_data::{Block, BlockDirection, block_properties::BlockProperties};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::{
    block::{BlockIsReplacing, pumpkin_block::PumpkinBlock},
    entity::{EntityBase, player::Player},
    server::Server,
    world::World,
};

type EndPortalFrameProperties = pumpkin_data::block_properties::EndPortalFrameLikeProperties;

#[pumpkin_block("minecraft:end_portal_frame")]
pub struct EndPortalFrameBlock;

#[async_trait]
impl PumpkinBlock for EndPortalFrameBlock {
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
        let mut end_portal_frame_props = EndPortalFrameProperties::default(block);
        end_portal_frame_props.facing = player.get_entity().get_horizontal_facing().opposite();

        end_portal_frame_props.to_state_id(block)
    }
}

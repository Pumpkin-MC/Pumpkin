use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

use crate::{
    block::pumpkin_block::{OnPlaceArgs, PumpkinBlock},
    entity::EntityBase,
};

type EndPortalFrameProperties = pumpkin_data::block_properties::EndPortalFrameLikeProperties;

#[pumpkin_block("minecraft:end_portal_frame")]
pub struct EndPortalFrameBlock;

#[async_trait]
impl PumpkinBlock for EndPortalFrameBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut end_portal_frame_props = EndPortalFrameProperties::default(args.block);
        end_portal_frame_props.facing = args.player.get_entity().get_horizontal_facing().opposite();

        end_portal_frame_props.to_state_id(args.block)
    }
}

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_macros::pumpkin_block;

use crate::{
    block::{BlockBehaviour, GetComparatorOutputArgs, OnPlaceArgs},
    entity::EntityBase,
};

type EndPortalFrameProperties = pumpkin_data::block_properties::EndPortalFrameLikeProperties;

#[pumpkin_block("minecraft:end_portal_frame")]
pub struct EndPortalFrameBlock;

impl BlockBehaviour for EndPortalFrameBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut end_portal_frame_props = EndPortalFrameProperties::default(args.block);
        end_portal_frame_props.facing = args.player.get_entity().get_horizontal_facing().opposite();

        end_portal_frame_props.to_state_id(args.block)
    }

    // Vanilla `EndPortalFrameBlock.getAnalogOutputSignal`: full signal once the eye of ender
    // sits in the frame.
    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        let state_id = args.world.get_block_state_id(args.position);
        let props = EndPortalFrameProperties::from_state_id(state_id, args.block);
        Some(if props.eye { 15 } else { 0 })
    }
}

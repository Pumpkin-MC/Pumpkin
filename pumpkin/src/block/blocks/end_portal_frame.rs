use pumpkin_data::block_properties::BlockProperties;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

use crate::{
    block::{BlockBehaviour, BlockFuture, OnPlaceArgs},
    entity::EntityBase,
};

type EndPortalFrameProperties = pumpkin_data::block_properties::EndPortalFrameLikeProperties;

#[pumpkin_block("minecraft:end_portal_frame")]
pub struct EndPortalFrameBlock;

impl BlockBehaviour for EndPortalFrameBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut end_portal_frame_props = EndPortalFrameProperties::default(args.block);
            end_portal_frame_props.facing =
                args.player.get_entity().get_horizontal_facing().opposite();

            end_portal_frame_props.to_state_id(args.block)
        })
    }
}

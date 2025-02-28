use pumpkin_data::packet::serverbound::{PLAY_PICK_ITEM_FROM_BLOCK, PLAY_PICK_ITEM_FROM_ENTITY};
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;
use serde::Deserialize;

#[derive(Deserialize)]
#[packet(PLAY_PICK_ITEM_FROM_BLOCK)]
pub struct SPickItemFromBlock {
    pub pos: BlockPos,
    pub include_data: bool,
}

#[derive(Deserialize)]
#[packet(PLAY_PICK_ITEM_FROM_ENTITY)]
pub struct SPickItemFromEntity {
    pub id: i32,
    pub include_data: bool,
}

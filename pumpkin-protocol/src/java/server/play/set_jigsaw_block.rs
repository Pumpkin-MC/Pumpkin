use pumpkin_data::packet::serverbound::PLAY_SET_JIGSAW_BLOCK;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;
use serde::Deserialize;

use crate::codec::var_int::VarInt;

#[derive(Deserialize)]
#[java_packet(PLAY_SET_JIGSAW_BLOCK)]
pub struct SSetJigsawBlock {
    pub pos: BlockPos,
    pub name: String,
    pub target: String,
    pub pool: String,
    pub final_state: String,
    pub joint: String,
    pub selection_priority: VarInt,
    pub placement_priority: VarInt,
}

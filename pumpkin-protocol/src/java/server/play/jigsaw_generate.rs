use pumpkin_data::packet::serverbound::PLAY_JIGSAW_GENERATE;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;
use serde::Deserialize;

use crate::codec::var_int::VarInt;

#[derive(Deserialize)]
#[java_packet(PLAY_JIGSAW_GENERATE)]
pub struct SJigsawGenerate {
    pub pos: BlockPos,
    pub levels: VarInt,
    pub keep_jigsaws: bool,
}

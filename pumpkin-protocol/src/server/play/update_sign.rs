use bytes::Buf;
use pumpkin_data::packet::serverbound::PLAY_SIGN_UPDATE;
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{
    ServerPacket,
    bytebuf::{ByteBuf, ReadingError},
};

#[packet(PLAY_SIGN_UPDATE)]
pub struct SUpdateSign {
    pub location: BlockPos,
    pub is_front_text: bool,
    pub line_1: String,
    pub line_2: String,
    pub line_3: String,
    pub line_4: String,
}

impl ServerPacket for SUpdateSign {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        Ok(Self {
            location: BlockPos::from_i64(bytebuf.try_get_i64()?),
            is_front_text: bytebuf.try_get_bool()?,
            line_1: bytebuf.try_get_string_len(386)?,
            line_2: bytebuf.try_get_string_len(386)?,
            line_3: bytebuf.try_get_string_len(386)?,
            line_4: bytebuf.try_get_string_len(386)?,
        })
    }
}

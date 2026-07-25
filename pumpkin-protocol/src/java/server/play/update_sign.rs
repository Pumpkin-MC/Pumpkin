use pumpkin_data::packet::serverbound::PLAY_SIGN_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};

#[java_packet(PLAY_SIGN_UPDATE)]
pub struct SUpdateSign<'a> {
    pub location: BlockPos,
    pub is_front_text: bool,
    pub line_1: &'a str,
    pub line_2: &'a str,
    pub line_3: &'a str,
    pub line_4: &'a str,
}

const MAX_LINE_LENGTH: usize = 386;

impl<'a> ServerPacket<'a> for SUpdateSign<'a> {
    fn read(read: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            location: BlockPos::from_i64(read.get_i64_be()?),
            is_front_text: read.get_bool()?,
            line_1: read.get_str_bounded_borrowed(MAX_LINE_LENGTH)?,
            line_2: read.get_str_bounded_borrowed(MAX_LINE_LENGTH)?,
            line_3: read.get_str_bounded_borrowed(MAX_LINE_LENGTH)?,
            line_4: read.get_str_bounded_borrowed(MAX_LINE_LENGTH)?,
        })
    }
}

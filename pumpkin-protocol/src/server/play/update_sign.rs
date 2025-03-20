use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_SIGN_UPDATE;
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
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

const MAX_LINE_LENGTH: usize = 386;

impl ServerPacket for SUpdateSign {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;

        Ok(Self {
            location: BlockPos::from_i64(read.get_i64_be()?),
            is_front_text: read.get_bool()?,
            line_1: read.get_string_bounded(MAX_LINE_LENGTH)?,
            line_2: read.get_string_bounded(MAX_LINE_LENGTH)?,
            line_3: read.get_string_bounded(MAX_LINE_LENGTH)?,
            line_4: read.get_string_bounded(MAX_LINE_LENGTH)?,
        })
    }
}

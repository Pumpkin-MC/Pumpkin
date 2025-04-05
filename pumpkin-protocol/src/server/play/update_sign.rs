use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_SIGN_UPDATE;
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;
use pumpkin_config::BASIC_CONFIG;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[packet(PLAY_SIGN_UPDATE)]
pub struct SUpdateSign {
    pub location: BlockPos,
    pub is_front_text: bool,
    pub line_1: String, // 384
    pub line_2: String, // 384
    pub line_3: String, // 384
    pub line_4: String, // 384
}

impl ServerPacket for SUpdateSign {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;

        let max_line_length: usize = if BASIC_CONFIG.allow_impossible_actions {
            386
        } else {
            15
        };

        Ok(Self {
            location: BlockPos::from_i64(read.get_i64_be()?),
            is_front_text: read.get_bool()?,
            line_1: read.get_string_bounded(max_line_length)?,
            line_2: read.get_string_bounded(max_line_length)?,
            line_3: read.get_string_bounded(max_line_length)?,
            line_4: read.get_string_bounded(max_line_length)?,
        })
    }
}

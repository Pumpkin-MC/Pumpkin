use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_SIGN_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(PLAY_SIGN_UPDATE)]
pub struct SUpdateSign {
    pub location: BlockPos,
    pub is_front_text: bool,
    pub line_1: Box<str>,
    pub line_2: Box<str>,
    pub line_3: Box<str>,
    pub line_4: Box<str>,
}

const MAX_LINE_LENGTH: usize = 386;

impl ServerPacket for SUpdateSign {
    fn read(mut read: impl Read, _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            location: BlockPos::from_i64(read.get_i64_be()?),
            is_front_text: read.get_bool()?,
            line_1: read.get_str_bounded(MAX_LINE_LENGTH)?,
            line_2: read.get_str_bounded(MAX_LINE_LENGTH)?,
            line_3: read.get_str_bounded(MAX_LINE_LENGTH)?,
            line_4: read.get_str_bounded(MAX_LINE_LENGTH)?,
        })
    }
}

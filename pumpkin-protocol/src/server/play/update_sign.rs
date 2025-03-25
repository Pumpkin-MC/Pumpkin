use std::{borrow::Cow, io::Read};

use pumpkin_data::packet::serverbound::PLAY_SIGN_UPDATE;
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[packet(PLAY_SIGN_UPDATE)]
pub struct SUpdateSign<'a> {
    pub location: BlockPos,
    pub is_front_text: bool,
    pub line_1: Cow<'a, str>,
    pub line_2: Cow<'a, str>,
    pub line_3: Cow<'a, str>,
    pub line_4: Cow<'a, str>,
}

const MAX_LINE_LENGTH: usize = 386;

impl ServerPacket for SUpdateSign<'_> {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;

        Ok(Self {
            location: BlockPos::from_i64(read.get_i64_be()?),
            is_front_text: read.get_bool()?,
            line_1: Cow::Owned(read.get_string_bounded(MAX_LINE_LENGTH)?),
            line_2: Cow::Owned(read.get_string_bounded(MAX_LINE_LENGTH)?),
            line_3: Cow::Owned(read.get_string_bounded(MAX_LINE_LENGTH)?),
            line_4: Cow::Owned(read.get_string_bounded(MAX_LINE_LENGTH)?),
        })
    }
}

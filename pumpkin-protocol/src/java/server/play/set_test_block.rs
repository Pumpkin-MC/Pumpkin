use pumpkin_data::packet::serverbound::PLAY_SET_TEST_BLOCK;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Read;

#[java_packet(PLAY_SET_TEST_BLOCK)]
pub struct SSetTestBlock<'a> {
    pub position: BlockPos,
    pub mode: TestBlockMode,
    pub message: &'a str,
}

impl<'a> ServerPacket<'a> for SSetTestBlock<'a> {
    fn read(
        mut bytebuf: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self {
            position: BlockPos::from_i64(bytebuf.get_i64_be()?),
            mode: TestBlockMode::read(&mut bytebuf)?,
            message: bytebuf.get_str_borrowed()?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}

impl TestBlockMode {
    fn read(bytebuf: &mut impl Read) -> Result<Self, ReadingError> {
        match bytebuf.get_var_int()?.0 {
            0 => Ok(Self::Start),
            1 => Ok(Self::Log),
            2 => Ok(Self::Fail),
            3 => Ok(Self::Accept),
            _ => Err(ReadingError::Message("Invalid TestBlockMode".to_string())),
        }
    }
}

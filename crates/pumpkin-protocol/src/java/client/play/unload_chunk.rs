use std::io::Write;

use pumpkin_data::packet::clientbound::play::FORGET_LEVEL_CHUNK;
use pumpkin_macros::java_packet;

use crate::{
    ClientPacket, ServerPacket,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

#[java_packet(FORGET_LEVEL_CHUNK)]
pub struct CUnloadChunk {
    pub x: i32,
    pub z: i32,
}

impl CUnloadChunk {
    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

impl ClientPacket for CUnloadChunk {
    fn write_packet_data(&self, mut write: impl Write) -> Result<(), WritingError> {
        {
            let chunk_key = ((self.z as i64) << 32) | ((self.x as u32) as i64);
            write.write_i64_be(chunk_key)?;
        };
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CUnloadChunk {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let (x, z) = {
            let chunk_key = bytebuf.get_i64_be()?;
            let x = chunk_key as i32;
            let z = (chunk_key >> 32) as i32;
            (x, z)
        };
        Ok(Self { x, z })
    }
}

use std::io::{Read, Write};

use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};

#[derive(Clone, Copy)]
pub struct U24(pub u32);

impl U24 {
    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let a = read.get_u8_le()?;
        let b = read.get_u8_le()?;
        let c = read.get_u8_le()?;
        Ok(U24(u32::from_le_bytes([a, b, c, 0])))
    }

    pub fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        let data = self.0.to_le_bytes();
        write.write_u8(data[0])?;
        write.write_u8(data[1])?;
        write.write_u8(data[2])?;
        Ok(())
    }
}

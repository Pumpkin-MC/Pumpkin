use std::io::{self, Write};

use bytes::Buf;
use thiserror::Error;

use crate::VarLongType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarLong(pub VarLongType);

impl VarLong {
    /// The maximum number of bytes a `VarInt` could occupy when read from and
    /// written to the Minecraft protocol.
    pub const MAX_SIZE: usize = 10;

    /// Returns the exact number of bytes this varint will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    pub const fn written_size(self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    pub fn encode(&self, mut w: impl Write) -> Result<(), io::Error> {
        let mut x = self.0 as u64;
        loop {
            let byte = (x & 0x7F) as u8;
            x >>= 7;
            if x == 0 {
                w.write_all(&[byte])?;
                break;
            }
            w.write_all(&[byte | 0x80])?;
        }
        Ok(())
    }

    pub fn decode(r: &mut &[u8]) -> Result<Self, VarLongDecodeError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE {
            if !r.has_remaining() {
                return Err(VarLongDecodeError::Incomplete);
            }
            let byte = r.get_u8();
            val |= (i64::from(byte) & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(VarLong(val));
            }
        }
        Err(VarLongDecodeError::TooLarge)
    }
}

impl From<i64> for VarLong {
    fn from(value: i64) -> Self {
        VarLong(value)
    }
}

impl From<u32> for VarLong {
    fn from(value: u32) -> Self {
        VarLong(value as i64)
    }
}

impl From<u8> for VarLong {
    fn from(value: u8) -> Self {
        VarLong(value as i64)
    }
}

impl From<usize> for VarLong {
    fn from(value: usize) -> Self {
        VarLong(value as i64)
    }
}

impl From<VarLong> for i64 {
    fn from(value: VarLong) -> Self {
        value.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Error)]
pub enum VarLongDecodeError {
    #[error("incomplete VarLong decode")]
    Incomplete,
    #[error("VarLong is too large")]
    TooLarge,
}

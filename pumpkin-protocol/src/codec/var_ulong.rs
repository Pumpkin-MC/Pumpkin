use std::{
    io::{Error, Read, Write},
    num::NonZeroUsize,
    ops::Deref,
};

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
};

use crate::{
    WritingError,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError},
    serial::{PacketRead, PacketWrite},
};

pub type VarULongType = u64;

/**
 * A variable-length long type used by the Minecraft network protocol.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VarULong(pub VarULongType);

impl VarULong {
    /// The maximum number of bytes a `VarULong` can occupy.
    const MAX_SIZE: NonZeroUsize = NonZeroUsize::new(10).unwrap();

    /// Returns the exact number of bytes this `VarLong` will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    #[must_use]
    pub const fn written_size(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    pub fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        let mut x = self.0;
        loop {
            let byte = (x & 0x7F) as u8;
            x >>= 7;
            if x == 0 {
                write.write_u8(byte)?;
                break;
            }
            write.write_u8(byte | 0x80)?;
        }

        Ok(())
    }

    // TODO: Validate that the first byte will not overflow a i64
    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.get_u8()?;
            val |= (u64::from(byte) & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(Self(val));
            }
        }
        Err(ReadingError::TooLarge("VarLong".to_owned()))
    }
}

impl From<u64> for VarULong {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<u32> for VarULong {
    fn from(value: u32) -> Self {
        Self(u64::from(value))
    }
}

impl From<u8> for VarULong {
    fn from(value: u8) -> Self {
        Self(u64::from(value))
    }
}

impl From<usize> for VarULong {
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl From<VarULong> for u64 {
    fn from(value: VarULong) -> Self {
        value.0
    }
}

impl AsRef<u64> for VarULong {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl Deref for VarULong {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for VarULong {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = self.0;
        let mut buf = Vec::new();

        while value > 0x7F {
            buf.push(value as u8 | 0x80);
            value >>= 7;
        }

        buf.push(value as u8);

        serializer.serialize_bytes(&buf)
    }
}

impl<'de> Deserialize<'de> for VarULong {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VarLongVisitor;

        impl<'de> Visitor<'de> for VarLongVisitor {
            type Value = VarULong;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut val = 0;
                for i in 0..VarULong::MAX_SIZE.get() {
                    if let Some(byte) = seq.next_element::<u8>()? {
                        val |= (u64::from(byte) & 0b01111111) << (i * 7);
                        if byte & 0b10000000 == 0 {
                            return Ok(VarULong(val));
                        }
                    } else {
                        break;
                    }
                }
                Err(de::Error::custom("VarInt was too large"))
            }
        }

        deserializer.deserialize_seq(VarLongVisitor)
    }
}

impl PacketWrite for VarULong {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let mut x = self.0;
        loop {
            let byte = (x & 0x7F) as u8;
            x >>= 7;
            if x == 0 {
                byte.write(writer)?;
                break;
            }
            (byte | 0x80).write(writer)?;
        }

        Ok(())
    }
}

impl PacketRead for VarULong {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = u8::read(reader)?;
            val |= (u64::from(byte) & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(Self(val));
            }
        }
        Err(Error::other("Invalid VarUInt"))
    }
}

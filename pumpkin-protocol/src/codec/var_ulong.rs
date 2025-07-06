use std::{
    io::{Read, Write},
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
};

pub type VarULongType = u64;

/**
 * A variable-length long type used by the Minecraft network protocol.
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarULong(pub VarULongType);

impl VarULong {
    /// The maximum number of bytes a `VarULong` can occupy.
    const MAX_SIZE: NonZeroUsize = NonZeroUsize::new(10).unwrap();

    /// Returns the exact number of bytes this VarLong will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    pub fn written_size(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    pub fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        let mut x = self.0;
        for _ in 0..Self::MAX_SIZE.get() {
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
                return Ok(VarULong(val));
            }
        }
        Err(ReadingError::TooLarge("VarLong".to_string()))
    }
}

impl From<u64> for VarULong {
    fn from(value: u64) -> Self {
        VarULong(value)
    }
}

impl From<u32> for VarULong {
    fn from(value: u32) -> Self {
        VarULong(value as u64)
    }
}

impl From<u8> for VarULong {
    fn from(value: u8) -> Self {
        VarULong(value as u64)
    }
}

impl From<usize> for VarULong {
    fn from(value: usize) -> Self {
        VarULong(value as u64)
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VarLongVisitor;

        impl<'de> Visitor<'de> for VarLongVisitor {
            type Value = VarULong;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
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

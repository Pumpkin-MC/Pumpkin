use std::{num::NonZeroUsize, ops::Deref};

use super::{Codec, DecodeError};
use bytes::{Buf, BufMut};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
};

pub type VarIntType = i32;

/**
 * A variable-length integer type used by the Minecraft network protocol.
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarInt(pub VarIntType);

impl Codec<Self> for VarInt {
    /// The maximum number of bytes a `VarInt` can occupy.
    const MAX_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(5) };

    /// Returns the exact number of bytes this varint will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    fn written_size(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    // Adapted from VarInt-Simd encode
    // https://github.com/as-com/varint-simd/blob/0f468783da8e181929b01b9c6e9f741c1fe09825/src/encode/mod.rs#L71
    fn encode(&self, write: &mut impl BufMut) {
        let x = self.0 as u64;
        let stage1 = (x & 0x000000000000007f)
            | ((x & 0x0000000000003f80) << 1)
            | ((x & 0x00000000001fc000) << 2)
            | ((x & 0x000000000fe00000) << 3)
            | ((x & 0x00000000f0000000) << 4);

        let leading = stage1.leading_zeros();

        let unused_bytes = (leading - 1) >> 3;
        let bytes_needed = 8 - unused_bytes;

        // set all but the last MSBs
        let msbs = 0x8080808080808080;
        let msbmask = 0xffffffffffffffff >> (((8 - bytes_needed + 1) << 3) - 1);

        let merged = stage1 | (msbs & msbmask);
        let bytes = merged.to_le_bytes();
        write.put(&bytes[..bytes_needed as usize]);
    }

    fn decode(read: &mut impl Buf) -> Result<Self, DecodeError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            if !read.has_remaining() {
                return Err(DecodeError::Incomplete);
            }
            let byte = read.get_u8();
            val |= (i32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(VarInt(val));
            }
        }
        Err(DecodeError::TooLarge)
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        VarInt(value)
    }
}

impl From<u32> for VarInt {
    fn from(value: u32) -> Self {
        VarInt(value as i32)
    }
}

impl From<u8> for VarInt {
    fn from(value: u8) -> Self {
        VarInt(value as i32)
    }
}

impl From<usize> for VarInt {
    fn from(value: usize) -> Self {
        VarInt(value as i32)
    }
}

impl From<VarInt> for i32 {
    fn from(value: VarInt) -> Self {
        value.0
    }
}

impl AsRef<i32> for VarInt {
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

impl Deref for VarInt {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for VarInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut value = self.0 as u32;
        let mut buf = Vec::new();

        while value > 0x7F {
            buf.put_u8(value as u8 | 0x80);
            value >>= 7;
        }

        buf.put_u8(value as u8);

        serializer.serialize_bytes(&buf)
    }
}

impl<'de> Deserialize<'de> for VarInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VarIntVisitor;

        impl<'de> Visitor<'de> for VarIntVisitor {
            type Value = VarInt;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut val = 0;
                for i in 0..VarInt::MAX_SIZE.get() {
                    if let Some(byte) = seq.next_element::<u8>()? {
                        val |= (i32::from(byte) & 0b01111111) << (i * 7);
                        if byte & 0b10000000 == 0 {
                            return Ok(VarInt(val));
                        }
                    } else {
                        break;
                    }
                }
                Err(serde::de::Error::custom("VarInt was too large"))
            }
        }

        deserializer.deserialize_seq(VarIntVisitor)
    }
}

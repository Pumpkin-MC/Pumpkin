use std::{
    io::{ErrorKind, Read, Write},
    num::NonZeroUsize,
    ops::Deref,
};

use bytes::BufMut;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};

pub type VarIntType = i32;

/**
 * A variable-length integer type used by the Minecraft network protocol.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VarInt(pub VarIntType);

impl VarInt {
    /// The maximum number of bytes a `VarInt` can occupy.
    const MAX_SIZE: NonZeroUsize = NonZeroUsize::new(5).unwrap();

    /// Returns the exact number of bytes this VarInt will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    pub fn written_size(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    pub fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        let mut val = self.0;
        for _ in 0..Self::MAX_SIZE.get() {
            let b: u8 = val as u8 & 0b01111111;
            val >>= 7;
            write.write_u8_be(if val == 0 { b } else { b | 0b10000000 })?;
            if val == 0 {
                break;
            }
        }
        Ok(())
    }

    // TODO: Validate that the first byte will not overflow a i32
    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.get_u8_be()?;
            val |= (i32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(VarInt(val));
            }
        }
        Err(ReadingError::TooLarge("VarInt".to_string()))
    }
}

impl VarInt {
    pub async fn decode_async(read: &mut (impl AsyncRead + Unpin)) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.read_u8().await.map_err(|err| {
                if i == 0 && matches!(err.kind(), ErrorKind::UnexpectedEof) {
                    ReadingError::CleanEOF("VarInt".to_string())
                } else {
                    ReadingError::Incomplete(err.to_string())
                }
            })?;
            val |= (i32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(VarInt(val));
            }
        }
        Err(ReadingError::TooLarge("VarInt".to_string()))
    }

    pub async fn encode_async(
        &self,
        write: &mut (impl AsyncWrite + Unpin),
    ) -> Result<(), WritingError> {
        let mut val = self.0;
        for _ in 0..Self::MAX_SIZE.get() {
            let b: u8 = val as u8 & 0b01111111;
            val >>= 7;
            write
                .write_u8(if val == 0 { b } else { b | 0b10000000 })
                .await
                .map_err(WritingError::IoError)?;
            if val == 0 {
                break;
            }
        }
        Ok(())
    }
}

// Macros are needed because traits over generics succccccccccck
macro_rules! gen_from {
    ($ty: ty) => {
        impl From<$ty> for VarInt {
            fn from(value: $ty) -> Self {
                VarInt(value.into())
            }
        }
    };
}

gen_from!(i8);
gen_from!(u8);
gen_from!(i16);
gen_from!(u16);
gen_from!(i32);

macro_rules! gen_try_from {
    ($ty: ty) => {
        impl TryFrom<$ty> for VarInt {
            type Error = <i32 as TryFrom<$ty>>::Error;

            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                Ok(VarInt(value.try_into()?))
            }
        }
    };
}

gen_try_from!(u32);
gen_try_from!(i64);
gen_try_from!(u64);
gen_try_from!(isize);
gen_try_from!(usize);

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

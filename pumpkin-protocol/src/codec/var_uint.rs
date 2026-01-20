use std::{
    io::{Error, ErrorKind, Read, Write},
    num::NonZeroUsize,
};

use bytes::BufMut;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
    serial::{PacketRead, PacketWrite},
};

pub type VarUIntType = u32;

/**
 * A variable-length integer type used by the Minecraft network protocol.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VarUInt(pub VarUIntType);

impl VarUInt {
    /// The maximum number of bytes a `VarUInt` can occupy.
    const MAX_SIZE: NonZeroUsize = NonZeroUsize::new(5).unwrap();

    /// Returns the exact number of bytes this `VarUInt` will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    #[must_use] 
    pub fn written_size(&self) -> usize {
        (32 - self.0.leading_zeros() as usize).max(1).div_ceil(7)
    }

    pub fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        let mut val = self.0;
        loop {
            let mut byte = (val & 0x7F) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0x80;
            }
            write.write_u8(byte)?;
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
            let byte = read.get_u8()?;
            val |= (u32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(Self(val));
            }
        }
        Err(ReadingError::TooLarge("VarUInt".to_owned()))
    }
}

impl VarUInt {
    pub async fn decode_async(read: &mut (impl AsyncRead + Unpin)) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.read_u8().await.map_err(|err| {
                if i == 0 && matches!(err.kind(), ErrorKind::UnexpectedEof) {
                    ReadingError::CleanEOF("VarInt".to_owned())
                } else {
                    ReadingError::Incomplete(err.to_string())
                }
            })?;
            val |= (u32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(Self(val));
            }
        }
        Err(ReadingError::TooLarge("VarUInt".to_owned()))
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
        impl From<$ty> for VarUInt {
            fn from(value: $ty) -> Self {
                VarUInt(value as u32)
            }
        }
    };
}

gen_from!(i8);
gen_from!(u8);
gen_from!(i16);
gen_from!(u16);
gen_from!(u32);

macro_rules! gen_from {
    ($ty: ty) => {
        impl From<$ty> for VarUInt {
            fn from(value: $ty) -> Self {
                VarUInt(value as u32)
            }
        }
    };
}

gen_from!(i32);
gen_from!(i64);
gen_from!(u64);
gen_from!(isize);
gen_from!(usize);

impl Serialize for VarUInt {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = self.0;
        let mut buf = Vec::with_capacity(5);

        while value > 0x7F {
            buf.put_u8(value as u8 | 0x80);
            value >>= 7;
        }

        buf.put_u8(value as u8);

        serializer.serialize_bytes(&buf)
    }
}

impl<'de> Deserialize<'de> for VarUInt {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VarIntVisitor;

        impl<'de> Visitor<'de> for VarIntVisitor {
            type Value = VarUInt;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut val = 0;
                for i in 0..VarUInt::MAX_SIZE.get() {
                    if let Some(byte) = seq.next_element::<u8>()? {
                        val |= (u32::from(byte) & 0b01111111) << (i * 7);
                        if byte & 0b10000000 == 0 {
                            return Ok(VarUInt(val));
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

impl PacketWrite for VarUInt {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let mut val = self.0;
        loop {
            let mut byte = (val & 0x7F) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0x80;
            }
            byte.write(writer)?;
            if val == 0 {
                break;
            }
        }
        Ok(())
    }
}

impl PacketRead for VarUInt {
    fn read<W: Read>(reader: &mut W) -> Result<Self, Error> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = u8::read(reader)?;
            val |= (u32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(Self(val));
            }
        }
        Err(Error::new(ErrorKind::InvalidData, ""))
    }
}

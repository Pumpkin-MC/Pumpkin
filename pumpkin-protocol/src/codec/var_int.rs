use std::{
    io::{Error, ErrorKind, Read, Write},
    num::NonZeroUsize,
    ops::Deref,
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

pub type VarIntType = i32;

/**
 * A variable-length integer type used by the Minecraft network protocol.
 */
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VarInt(pub VarIntType);

impl VarInt {
    /// The maximum number of bytes a `VarInt` can occupy.
    const MAX_SIZE: NonZeroUsize = NonZeroUsize::new(5).unwrap();

    /// Returns the exact number of bytes this `VarInt` will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    #[must_use]
    pub const fn written_size(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    pub fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        // Must cast to u32 to prevent infinite loops on negative i32s
        let mut val = self.0 as u32;

        while val > 0x7F {
            write.write_u8((val as u8) | 0x80)?;
            val >>= 7;
        }

        write.write_u8(val as u8)?;
        Ok(())
    }

    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.get_u8()?;
            // On the 5th byte (i=4), only the lower 4 bits are valid (bits 28-31 of i32).
            // Upper bits would overflow the i32 representation.
            if i == Self::MAX_SIZE.get() - 1 && byte & 0xF0 != 0 {
                return Err(ReadingError::TooLarge("VarInt".to_string()));
            }
            val |= (i32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(Self(val));
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
            // On the 5th byte (i=4), only the lower 4 bits are valid (bits 28-31 of i32).
            if i == Self::MAX_SIZE.get() - 1 && byte & 0xF0 != 0 {
                return Err(ReadingError::TooLarge("VarInt".to_string()));
            }
            val |= (i32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(Self(val));
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
            let b: u8 = val as u8 & 0b0111_1111;
            val >>= 7;
            write
                .write_u8(if val == 0 { b } else { b | 0b1000_0000 })
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
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VarIntVisitor;

        impl<'de> Visitor<'de> for VarIntVisitor {
            type Value = VarInt;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut val = 0;
                for i in 0..VarInt::MAX_SIZE.get() {
                    if let Some(byte) = seq.next_element::<u8>()? {
                        // On the 5th byte (i=4), only the lower 4 bits are valid.
                        if i == VarInt::MAX_SIZE.get() - 1 && byte & 0xF0 != 0 {
                            return Err(serde::de::Error::custom("VarInt was too large"));
                        }
                        val |= (i32::from(byte) & 0b0111_1111) << (i * 7);
                        if byte & 0b1000_0000 == 0 {
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

impl PacketWrite for VarInt {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let mut val = ((self.0 << 1) ^ (self.0 >> 31)) as u32;

        while val > 0x7F {
            ((val as u8 & 0x7F) | 0x80).write(writer)?;
            val >>= 7;
        }

        (val as u8).write(writer)?;
        Ok(())
    }
}

impl PacketRead for VarInt {
    fn read<W: Read>(read: &mut W) -> Result<Self, Error> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = u8::read(read)?;
            val |= (i32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(Self((val >> 1) ^ (val << 31)));
            }
        }
        Err(Error::new(ErrorKind::InvalidData, ""))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn varint_encode_decode_zero() {
        let mut buf = Vec::new();
        VarInt(0).encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x00]);
        let decoded = VarInt::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, VarInt(0));
    }

    #[test]
    fn varint_encode_decode_positive() {
        let mut buf = Vec::new();
        VarInt(1).encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x01]);
        let decoded = VarInt::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, VarInt(1));
    }

    #[test]
    fn varint_encode_decode_large() {
        let mut buf = Vec::new();
        VarInt(300).encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0xAC, 0x02]);
        let decoded = VarInt::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, VarInt(300));
    }

    #[test]
    fn varint_encode_decode_negative() {
        let mut buf = Vec::new();
        VarInt(-1).encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        let decoded = VarInt::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, VarInt(-1));
    }

    #[test]
    fn varint_encode_decode_max() {
        let mut buf = Vec::new();
        VarInt(i32::MAX).encode(&mut buf).unwrap();
        let decoded = VarInt::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, VarInt(i32::MAX));
    }

    #[test]
    fn varint_encode_decode_min() {
        let mut buf = Vec::new();
        VarInt(i32::MIN).encode(&mut buf).unwrap();
        let decoded = VarInt::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, VarInt(i32::MIN));
    }

    #[test]
    fn varint_overflow_rejected() {
        // 5th byte with upper bits set (0x10 = bit 4 set, would be bit 32 of i32)
        let buf = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x1F];
        let result = VarInt::decode(&mut Cursor::new(&buf));
        assert!(result.is_err());
    }

    #[test]
    fn varint_too_many_bytes() {
        // 6 continuation bytes — exceeds max size
        let buf = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x00];
        let result = VarInt::decode(&mut Cursor::new(&buf));
        assert!(result.is_err());
    }

    #[test]
    fn varint_written_size() {
        assert_eq!(VarInt(0).written_size(), 1);
        assert_eq!(VarInt(1).written_size(), 1);
        assert_eq!(VarInt(127).written_size(), 1);
        assert_eq!(VarInt(128).written_size(), 2);
        assert_eq!(VarInt(255).written_size(), 2);
        assert_eq!(VarInt(i32::MAX).written_size(), 5);
        assert_eq!(VarInt(-1).written_size(), 5);
    }

    #[test]
    fn varint_roundtrip_all_sizes() {
        let values = [
            0,
            1,
            127,
            128,
            16383,
            16384,
            2097151,
            2097152,
            268435455,
            268435456,
            i32::MAX,
            -1,
            i32::MIN,
        ];
        for &val in &values {
            let mut buf = Vec::new();
            VarInt(val).encode(&mut buf).unwrap();
            let decoded = VarInt::decode(&mut Cursor::new(&buf)).unwrap();
            assert_eq!(decoded, VarInt(val), "Roundtrip failed for {val}");
        }
    }
}

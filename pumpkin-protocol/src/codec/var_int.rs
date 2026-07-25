use std::{
    io::{Error, ErrorKind, Read, Write},
    num::NonZeroUsize,
    ops::Deref,
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

    // TODO: Validate that the first byte will not overflow a i32
    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.get_u8()?;
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
        // Cast to u32 so negative values terminate (matches `encode`).
        let mut val = self.0 as u32;
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
        let mut val: u32 = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = u8::read(read)?;
            val |= (u32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                // Bedrock VarInts are ZigZag-encoded: (val >> 1) ^ -(val & 1).
                // The shift must be logical (u32), not arithmetic.
                return Ok(Self(((val >> 1) as i32) ^ -((val & 1) as i32)));
            }
        }
        Err(Error::new(ErrorKind::InvalidData, "VarInt is too big"))
    }
}

#[cfg(test)]
mod zigzag_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn round_trip_zigzag_values() {
        for value in [i32::MIN, -100, -2, -1, 0, 1, 2, 63, 64, 100, i32::MAX] {
            let mut buf = Vec::new();
            VarInt(value).write(&mut buf).unwrap();
            let decoded = VarInt::read(&mut Cursor::new(buf)).unwrap();
            assert_eq!(decoded.0, value, "round trip failed for {value}");
        }
    }

    #[test]
    fn decodes_zigzag_wire_vectors() {
        // Single-byte ZigZag wire values.
        for (wire, expected) in [(0x00u8, 0i32), (0x01, -1), (0x02, 1), (0x03, -2)] {
            let decoded = VarInt::read(&mut Cursor::new(vec![wire])).unwrap();
            assert_eq!(decoded.0, expected, "wire {wire:#04x}");
        }
        // Five-byte encoding of i32::MIN (ZigZag 0xFFFF_FFFF).
        let decoded = VarInt::read(&mut Cursor::new(vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F])).unwrap();
        assert_eq!(decoded.0, i32::MIN);
    }

    #[test]
    fn encodes_zigzag_wire_vectors() {
        for (value, wire) in [
            (0i32, vec![0x00u8]),
            (-1, vec![0x01]),
            (1, vec![0x02]),
            (-2, vec![0x03]),
            (i32::MIN, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]),
        ] {
            let mut buf = Vec::new();
            VarInt(value).write(&mut buf).unwrap();
            assert_eq!(buf, wire, "encode {value}");
        }
    }

    #[test]
    fn rejects_too_many_bytes() {
        let err = VarInt::read(&mut Cursor::new(vec![0x80u8; 6])).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }
}

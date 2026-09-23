use std::{
    io::{Error, ErrorKind, Read, Write},
    num::NonZero,
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
    pub const MAX_SIZE: NonZero<usize> = NonZero::new(5).expect("5 is non-zero");

    #[must_use]
    #[inline]
    pub const fn new(value: VarUIntType) -> Self {
        Self(value)
    }

    /// Returns the exact number of bytes this `VarUInt` will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    #[must_use]
    #[inline]
    pub const fn written_size(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    #[inline]
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
    #[inline]
    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.get_u8()?;
            val |= (u32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                if i == Self::MAX_SIZE.get() - 1 && byte & 0x7F > 0x0F {
                    return Err(ReadingError::TooLarge("VarUInt".to_string()));
                }
                return Ok(Self(val));
            }
        }
        Err(ReadingError::TooLarge("VarUInt".to_string()))
    }
}

impl VarUInt {
    pub async fn decode_async(read: &mut (impl AsyncRead + Unpin)) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.read_u8().await.map_err(|err| {
                if i == 0
                    && matches!(
                        err.kind(),
                        ErrorKind::UnexpectedEof
                            | ErrorKind::ConnectionReset
                            | ErrorKind::ConnectionAborted
                            | ErrorKind::BrokenPipe
                    )
                {
                    ReadingError::CleanEOF("VarUInt".to_string())
                } else {
                    ReadingError::Incomplete(err.to_string())
                }
            })?;
            val |= (u32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                if i == Self::MAX_SIZE.get() - 1 && byte & 0x7F > 0x0F {
                    return Err(ReadingError::TooLarge("VarUInt".to_string()));
                }
                return Ok(Self(val));
            }
        }
        Err(ReadingError::TooLarge("VarUInt".to_string()))
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
                if i == Self::MAX_SIZE.get() - 1 && byte & 0x7F > 0x0F {
                    return Err(Error::new(ErrorKind::InvalidData, "VarUInt is too big"));
                }
                return Ok(Self(val));
            }
        }
        Err(Error::new(ErrorKind::InvalidData, ""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_rejects_final_byte_payload_overflow() {
        assert_eq!(
            VarUInt::decode(&mut [0xFF, 0xFF, 0xFF, 0xFF, 0x0F].as_slice()).unwrap(),
            VarUInt(u32::MAX)
        );
        assert!(VarUInt::decode(&mut [0x80, 0x80, 0x80, 0x80, 0x10].as_slice()).is_err());
        assert!(VarUInt::decode(&mut [0xFF, 0xFF, 0xFF, 0xFF, 0x7F].as_slice()).is_err());
        assert!(VarUInt::read(&mut [0x80, 0x80, 0x80, 0x80, 0x20].as_slice()).is_err());
    }

    #[tokio::test]
    async fn decode_async_rejects_final_byte_payload_overflow() {
        assert_eq!(
            VarUInt::decode_async(&mut [0xFF, 0xFF, 0xFF, 0xFF, 0x0F].as_slice())
                .await
                .unwrap(),
            VarUInt(u32::MAX)
        );
        assert!(
            VarUInt::decode_async(&mut [0x80, 0x80, 0x80, 0x80, 0x10].as_slice())
                .await
                .is_err()
        );
        assert!(
            VarUInt::decode_async(&mut [0xFF, 0xFF, 0xFF, 0xFF, 0x7F].as_slice())
                .await
                .is_err()
        );
    }
}

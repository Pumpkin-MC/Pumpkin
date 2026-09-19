use std::{
    io::{Error, Read, Write},
    num::NonZero,
    ops::Deref,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VarULong(pub VarULongType);

impl VarULong {
    pub const MAX_SIZE: NonZero<usize> = NonZero::new(10).expect("10 is non-zero");

    #[must_use]
    #[inline]
    pub const fn new(value: VarULongType) -> Self {
        Self(value)
    }

    /// Returns the exact number of bytes this `VarULong` will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    #[must_use]
    #[inline]
    pub const fn written_size(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (63 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    #[inline]
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

    #[inline]
    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.get_u8()?;
            if i == Self::MAX_SIZE.get() - 1 && (byte & 0x7F) > 0x01 {
                return Err(ReadingError::TooLarge("VarULong".to_string()));
            }
            val |= (u64::from(byte) & 0b0111_1111) << (i * 7);
            if byte & 0b1000_0000 == 0 {
                return Ok(Self(val));
            }
        }
        Err(ReadingError::TooLarge("VarULong".to_string()))
    }
}
macro_rules! gen_from {
    ($ty: ty) => {
        impl From<$ty> for VarULong {
            fn from(value: $ty) -> Self {
                VarULong(value.into())
            }
        }
    };
}
gen_from!(u8);
gen_from!(u16);
gen_from!(u32);
gen_from!(u64);

macro_rules! gen_try_from {
    ($ty: ty) => {
        impl TryFrom<$ty> for VarULong {
            type Error = <u64 as TryFrom<$ty>>::Error;

            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                Ok(VarULong(value.try_into()?))
            }
        }
    };
}
gen_try_from!(i32);
gen_try_from!(i64);
gen_try_from!(isize);
gen_try_from!(usize);

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
            if i == Self::MAX_SIZE.get() - 1 && (byte & 0x7F) > 0x01 {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "VarULong is too big (overflow)",
                ));
            }
            val |= (u64::from(byte) & 0b0111_1111) << (i * 7);
            if byte & 0b1000_0000 == 0 {
                return Ok(Self(val));
            }
        }
        Err(Error::other("Invalid VarUInt"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_ENCODING: [u8; 10] = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01];
    const OVERFLOW_ENCODING: [u8; 10] =
        [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02];
    const ALL_ONES_ENCODING: [u8; 10] =
        [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f];

    fn decode(bytes: &[u8]) -> Result<VarULong, ReadingError> {
        let mut reader = bytes;
        VarULong::decode(&mut reader)
    }

    fn packet_read(bytes: &[u8]) -> Result<VarULong, Error> {
        let mut reader = bytes;
        VarULong::read(&mut reader)
    }

    #[test]
    fn decodes_largest_valid_values() {
        assert_eq!(decode(&MAX_ENCODING).unwrap().0, u64::MAX);
        assert_eq!(
            decode(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00])
                .unwrap()
                .0,
            0x7fff_ffff_ffff_ffff
        );
    }

    #[test]
    fn rejects_overflowing_final_byte() {
        // The tenth byte only has a single payload bit left.
        assert!(decode(&OVERFLOW_ENCODING).is_err());
        assert!(decode(&ALL_ONES_ENCODING).is_err());
    }

    #[test]
    fn packet_read_rejects_overflowing_final_byte() {
        assert!(packet_read(&OVERFLOW_ENCODING).is_err());
        assert!(packet_read(&ALL_ONES_ENCODING).is_err());
        assert_eq!(packet_read(&MAX_ENCODING).unwrap().0, u64::MAX);
    }

    #[test]
    fn round_trips_extremes() {
        for value in [0, 1, u64::MAX, 0x7fff_ffff_ffff_ffff] {
            let mut encoded = Vec::new();
            VarULong(value).encode(&mut encoded).unwrap();
            assert_eq!(
                VarULong::decode(&mut encoded.as_slice()).unwrap(),
                VarULong(value)
            );
            assert_eq!(
                VarULong::read(&mut encoded.as_slice()).unwrap(),
                VarULong(value)
            );
        }
    }
}

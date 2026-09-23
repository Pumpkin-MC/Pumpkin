use std::{
    io::{Error, ErrorKind, Read, Write},
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

    // TODO: Validate that the first byte will not overflow a i64
    #[inline]
    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.get_u8()?;
            val |= (u64::from(byte) & 0b0111_1111) << (i * 7);
            if byte & 0b1000_0000 == 0 {
                if i == Self::MAX_SIZE.get() - 1 && byte & 0x7F > 0x01 {
                    return Err(ReadingError::TooLarge("VarULong".to_string()));
                }
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
            val |= (u64::from(byte) & 0b0111_1111) << (i * 7);
            if byte & 0b1000_0000 == 0 {
                if i == Self::MAX_SIZE.get() - 1 && byte & 0x7F > 0x01 {
                    return Err(Error::new(ErrorKind::InvalidData, "VarULong is too big"));
                }
                return Ok(Self(val));
            }
        }
        Err(Error::other("Invalid VarUInt"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_rejects_final_byte_payload_overflow() {
        let max = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        assert_eq!(
            VarULong::decode(&mut max.as_slice()).unwrap(),
            VarULong(u64::MAX)
        );

        let mut overflow = [0xFF; 10];
        overflow[9] = 0x7F;
        assert!(VarULong::decode(&mut overflow.as_slice()).is_err());

        let mut high_bit = [0x80; 10];
        high_bit[9] = 0x02;
        assert!(VarULong::decode(&mut high_bit.as_slice()).is_err());
    }

    #[test]
    fn packet_read_round_trips_and_rejects_overflow() {
        for value in [0, 1, u64::from(u32::MAX), u64::MAX] {
            let mut encoded = Vec::new();
            VarULong(value).write(&mut encoded).unwrap();
            assert_eq!(
                VarULong::read(&mut encoded.as_slice()).unwrap(),
                VarULong(value)
            );
        }

        let mut overflow = [0xFF; 10];
        overflow[9] = 0x03;
        assert!(VarULong::read(&mut overflow.as_slice()).is_err());
    }
}

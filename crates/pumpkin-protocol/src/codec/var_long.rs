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

pub type VarLongType = i64;

/**
 * A variable-length long type used by the Minecraft network protocol.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VarLong(pub VarLongType);

impl VarLong {
    pub const MAX_SIZE: NonZero<usize> = NonZero::new(10).expect("10 is non-zero");

    #[must_use]
    #[inline]
    pub const fn new(value: VarLongType) -> Self {
        Self(value)
    }

    /// Returns the exact number of bytes this `VarLong` will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    #[must_use]
    #[inline]
    pub const fn written_size(&self) -> usize {
        match self.0 as u64 {
            0 => 1,
            n => (63 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    #[inline]
    pub fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        let mut val = self.0 as u64;

        while val > 0x7F {
            write.write_u8((val as u8) | 0x80)?;
            val >>= 7;
        }

        write.write_u8(val as u8)?;
        Ok(())
    }

    #[inline]
    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.get_u8()?;
            if i == Self::MAX_SIZE.get() - 1 && (byte & 0x7F) > 0x01 {
                return Err(ReadingError::TooLarge("VarLong".to_string()));
            }
            val |= (i64::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(Self(val));
            }
        }
        Err(ReadingError::TooLarge("VarLong".to_string()))
    }
}
macro_rules! gen_from {
    ($ty: ty) => {
        impl From<$ty> for VarLong {
            fn from(value: $ty) -> Self {
                VarLong(value.into())
            }
        }
    };
}

gen_from!(u8);
gen_from!(u32);
gen_from!(i64);

impl From<usize> for VarLong {
    fn from(value: usize) -> Self {
        Self(value as i64)
    }
}

impl From<VarLong> for i64 {
    fn from(value: VarLong) -> Self {
        value.0
    }
}

impl AsRef<i64> for VarLong {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl Deref for VarLong {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PacketRead for VarLong {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut val: u64 = 0;
        let mut shift = 0;

        loop {
            let byte = u8::read(reader)?;
            if shift == 63 && (byte & 0x7F) > 0x01 {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "VarLong is too big (overflow)",
                ));
            }
            val |= ((byte & 0x7F) as u64) << shift;

            if (byte & 0x80) == 0 {
                break;
            }

            shift += 7;
            if shift >= 64 {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "VarLong is too big (overflow)",
                ));
            }
        }

        let decoded = ((val >> 1) as i64) ^ -((val & 1) as i64);

        Ok(Self(decoded))
    }
}

impl PacketWrite for VarLong {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let mut val = ((self.0 << 1) ^ (self.0 >> 63)) as u64;

        while val > 0x7F {
            ((val as u8 & 0x7F) | 0x80).write(writer)?;
            val >>= 7;
        }

        (val as u8).write(writer)?;
        Ok(())
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

    fn decode(bytes: &[u8]) -> Result<VarLong, ReadingError> {
        let mut reader = bytes;
        VarLong::decode(&mut reader)
    }

    fn packet_read(bytes: &[u8]) -> Result<VarLong, Error> {
        let mut reader = bytes;
        VarLong::read(&mut reader)
    }

    #[test]
    fn decodes_largest_valid_values() {
        assert_eq!(decode(&MAX_ENCODING).unwrap().0, -1);
        assert_eq!(
            decode(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00])
                .unwrap()
                .0,
            i64::MAX
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
        assert_eq!(packet_read(&MAX_ENCODING).unwrap().0, i64::MIN);
    }

    #[test]
    fn round_trips_extremes() {
        for value in [i64::MIN, -1, 0, 1, i64::MAX] {
            let mut zig_zag = Vec::new();
            VarLong(value).write(&mut zig_zag).unwrap();
            assert_eq!(
                VarLong::read(&mut zig_zag.as_slice()).unwrap(),
                VarLong(value)
            );

            let mut plain = Vec::new();
            VarLong(value).encode(&mut plain).unwrap();
            assert_eq!(
                VarLong::decode(&mut plain.as_slice()).unwrap(),
                VarLong(value)
            );
        }
    }
}

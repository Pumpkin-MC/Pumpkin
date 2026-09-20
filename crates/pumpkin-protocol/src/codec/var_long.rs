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

    // TODO: Validate that the first byte will not overflow a i64
    #[inline]
    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.get_u8()?;
            val |= (i64::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                if i == Self::MAX_SIZE.get() - 1 && byte & 0x7F > 0x01 {
                    return Err(ReadingError::TooLarge("VarLong".to_string()));
                }
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
            val |= ((byte & 0x7F) as u64) << shift;

            if (byte & 0x80) == 0 {
                if shift == 63 && (byte & 0x7F) > 0x01 {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "VarLong is too big (overflow)",
                    ));
                }
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

    #[test]
    fn decode_rejects_final_byte_payload_overflow() {
        let minus_one = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        assert_eq!(
            VarLong::decode(&mut minus_one.as_slice()).unwrap(),
            VarLong(-1)
        );

        let mut overflow = [0xFF; 10];
        overflow[9] = 0x7F;
        assert!(VarLong::decode(&mut overflow.as_slice()).is_err());

        let mut high_bit = [0x80; 10];
        high_bit[9] = 0x02;
        assert!(VarLong::decode(&mut high_bit.as_slice()).is_err());
    }

    #[test]
    fn packet_read_round_trips_and_rejects_overflow() {
        for value in [i64::MIN, -2, -1, 0, 1, 2, i64::MAX] {
            let mut encoded = Vec::new();
            VarLong(value).write(&mut encoded).unwrap();
            assert_eq!(
                VarLong::read(&mut encoded.as_slice()).unwrap(),
                VarLong(value)
            );
        }

        let mut overflow = [0xFF; 10];
        overflow[9] = 0x03;
        assert!(VarLong::read(&mut overflow.as_slice()).is_err());
    }
}

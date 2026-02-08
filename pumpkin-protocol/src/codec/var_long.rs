use std::{
    io::{Error, Read, Write},
    num::NonZeroUsize,
    ops::Deref,
};

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
};

use crate::{
    WritingError,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError},
    serial::PacketWrite,
};

pub type VarLongType = i64;

/**
 * A variable-length long type used by the Minecraft network protocol.
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarLong(pub VarLongType);

impl VarLong {
    /// The maximum number of bytes a `VarLong` can occupy.
    const MAX_SIZE: NonZeroUsize = NonZeroUsize::new(10).unwrap();

    pub fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        let mut val = self.0 as u64;

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
            // On the 10th byte (i=9), only the lowest bit is valid (bit 63 of i64).
            // Upper bits would overflow the i64 representation.
            if i == Self::MAX_SIZE.get() - 1 && byte & 0xFE != 0 {
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

impl From<i64> for VarLong {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<u32> for VarLong {
    fn from(value: u32) -> Self {
        Self(i64::from(value))
    }
}

impl From<u8> for VarLong {
    fn from(value: u8) -> Self {
        Self(i64::from(value))
    }
}

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

impl Serialize for VarLong {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = self.0 as u64;
        let mut buf = Vec::new();

        while value > 0x7F {
            buf.push(value as u8 | 0x80);
            value >>= 7;
        }

        buf.push(value as u8);

        serializer.serialize_bytes(&buf)
    }
}

impl<'de> Deserialize<'de> for VarLong {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VarLongVisitor;

        impl<'de> Visitor<'de> for VarLongVisitor {
            type Value = VarLong;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut val = 0;
                for i in 0..VarLong::MAX_SIZE.get() {
                    if let Some(byte) = seq.next_element::<u8>()? {
                        // On the 10th byte (i=9), only the lowest bit is valid.
                        if i == VarLong::MAX_SIZE.get() - 1 && byte & 0xFE != 0 {
                            return Err(de::Error::custom("VarLong was too large"));
                        }
                        val |= (i64::from(byte) & 0b0111_1111) << (i * 7);
                        if byte & 0b1000_0000 == 0 {
                            return Ok(VarLong(val));
                        }
                    } else {
                        break;
                    }
                }
                Err(de::Error::custom("VarLong was too large"))
            }
        }

        deserializer.deserialize_seq(VarLongVisitor)
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
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn varlong_encode_decode_zero() {
        let mut buf = Vec::new();
        VarLong(0).encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x00]);
        let decoded = VarLong::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, VarLong(0));
    }

    #[test]
    fn varlong_encode_decode_positive() {
        let mut buf = Vec::new();
        VarLong(1).encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x01]);
        let decoded = VarLong::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, VarLong(1));
    }

    #[test]
    fn varlong_encode_decode_negative() {
        let mut buf = Vec::new();
        VarLong(-1).encode(&mut buf).unwrap();
        let decoded = VarLong::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, VarLong(-1));
    }

    #[test]
    fn varlong_encode_decode_max() {
        let mut buf = Vec::new();
        VarLong(i64::MAX).encode(&mut buf).unwrap();
        let decoded = VarLong::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, VarLong(i64::MAX));
    }

    #[test]
    fn varlong_encode_decode_min() {
        let mut buf = Vec::new();
        VarLong(i64::MIN).encode(&mut buf).unwrap();
        let decoded = VarLong::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, VarLong(i64::MIN));
    }

    #[test]
    fn varlong_overflow_rejected() {
        // 10th byte with upper bits set (0x02 = bit 1 set, would be bit 64 of i64)
        let buf = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02];
        let result = VarLong::decode(&mut Cursor::new(&buf));
        assert!(result.is_err());
    }

    #[test]
    fn varlong_roundtrip_all_sizes() {
        let values = [
            0i64,
            1,
            127,
            128,
            -1,
            i64::MAX,
            i64::MIN,
            2147483647,
            -2147483648,
        ];
        for &val in &values {
            let mut buf = Vec::new();
            VarLong(val).encode(&mut buf).unwrap();
            let decoded = VarLong::decode(&mut Cursor::new(&buf)).unwrap();
            assert_eq!(decoded, VarLong(val), "Roundtrip failed for {val}");
        }
    }
}

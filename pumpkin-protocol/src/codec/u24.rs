use std::io::{Read, Write};

use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use serde::{
    Deserialize,
    de::{self, SeqAccess},
};

#[derive(Clone, Copy)]
pub struct U24(pub u32);

impl U24 {
    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let a = read.get_u8_le()?;
        let b = read.get_u8_le()?;
        let c = read.get_u8_le()?;
        Ok(U24(u32::from_le_bytes([a, b, c, 0])))
    }

    pub fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        let data = self.0 & 0xFFFFFF; // Get the internal u32 value
        write.write_u8_be((data & 0xFF) as u8)?;
        write.write_u8_be(((data >> 8) & 0xFF) as u8)?;
        write.write_u8_be(((data >> 16) & 0xFF) as u8)?;
        Ok(())
    }
}

impl<'de> Deserialize<'de> for U24 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = U24;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid u24")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut data: u32 = 0;

                // Read the first byte (LSB)
                data |= seq.next_element::<u8>()?.unwrap() as u32;

                // Read the second byte and shift it by 8 bits
                data |= (seq.next_element::<u8>()?.unwrap() as u32) << 8;

                // Read the third next_element and shift it by 16 bits
                data |= (seq.next_element::<u8>()?.unwrap() as u32) << 16;

                // Mask to ensure only the lower 24 bits are kept
                Ok(U24(data & 0xFFFFFF))
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

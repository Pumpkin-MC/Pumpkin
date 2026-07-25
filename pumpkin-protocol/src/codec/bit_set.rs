use std::io::Read;
use std::io::Write;

use crate::ReadingError;
use crate::WritingError;
use crate::ser::NetworkReadExt;
use crate::ser::NetworkWriteExt;

pub struct BitSet(pub Box<[i64]>);

impl BitSet {
    pub fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        write.write_var_int(&self.0.len().try_into().map_err(|_| {
            WritingError::Message(format!("{} isn't representable as a VarInt", self.0.len()))
        })?)?;

        for b in &self.0 {
            write.write_i64_be(*b)?;
        }

        Ok(())
    }

    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        /// Bound for a decoded bit set's long count. Vanilla bit sets (chunk, light,
        /// chat acknowledgement) are far smaller; an uncapped `VarInt` length
        /// pre-allocates attacker-sized memory.
        const MAX_BITSET_LONGS: i32 = 4096;

        // Read length
        let length = read.get_var_int()?;
        if length.0 < 0 || length.0 > MAX_BITSET_LONGS {
            return Err(ReadingError::TooLarge(format!(
                "BitSet length {} exceeds limit {MAX_BITSET_LONGS}",
                length.0
            )));
        }
        let mut array: Vec<i64> = Vec::with_capacity(length.0 as usize);
        for _ in 0..length.0 {
            let long = read.get_i64_be()?;
            array.push(long);
        }
        Ok(Self(array.into_boxed_slice()))
    }
}

#[cfg(test)]
mod alloc_cap_tests {
    use super::*;
    use crate::codec::var_int::VarInt;
    use std::io::Cursor;

    #[test]
    fn rejects_oversize_length() {
        let mut buf = Vec::new();
        VarInt(4097).encode(&mut buf).unwrap();

        let result = BitSet::decode(&mut Cursor::new(buf));
        assert!(matches!(result, Err(ReadingError::TooLarge(_))));
    }

    #[test]
    fn rejects_negative_length() {
        let mut buf = Vec::new();
        VarInt(-1).encode(&mut buf).unwrap();

        let result = BitSet::decode(&mut Cursor::new(buf));
        assert!(matches!(result, Err(ReadingError::TooLarge(_))));
    }

    #[test]
    fn round_trips_small_bit_set() {
        let bit_set = BitSet(Box::new([i64::MIN, 0, i64::MAX]));
        let mut buf = Vec::new();
        bit_set.encode(&mut buf).unwrap();

        let decoded = BitSet::decode(&mut Cursor::new(buf)).unwrap();
        assert_eq!(decoded.0, bit_set.0);
    }
}

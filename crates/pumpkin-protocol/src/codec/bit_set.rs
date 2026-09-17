use std::io::Read;
use std::io::Write;

use crate::ReadingError;
use crate::WritingError;
use crate::ser::NetworkReadExt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BitSet(pub Box<[i64]>);

impl BitSet {
    #[must_use]
    pub fn from_u64(val: u64) -> Self {
        Self(Box::new([val as i64]))
    }

    #[must_use]
    pub fn from_i64(val: i64) -> Self {
        Self(Box::new([val]))
    }

    #[must_use]
    pub fn from_longs(longs: Vec<i64>) -> Self {
        Self(longs.into_boxed_slice())
    }

    #[must_use]
    pub fn as_u64(&self) -> u64 {
        self.0.first().copied().unwrap_or(0) as u64
    }

    #[must_use]
    pub fn as_i64(&self) -> i64 {
        self.0.first().copied().unwrap_or(0)
    }

    #[must_use]
    pub fn get_bit(&self, index: usize) -> bool {
        let word_idx = index / 64;
        let bit_idx = index % 64;
        self.0
            .get(word_idx)
            .is_some_and(|&w| (w & (1i64 << bit_idx)) != 0)
    }

    pub fn set_bit(&mut self, index: usize, val: bool) {
        let word_idx = index / 64;
        let bit_idx = index % 64;
        if word_idx >= self.0.len() {
            let mut vec = self.0.to_vec();
            vec.resize(word_idx + 1, 0);
            self.0 = vec.into_boxed_slice();
        }
        if val {
            self.0[word_idx] |= 1i64 << bit_idx;
        } else {
            self.0[word_idx] &= !(1i64 << bit_idx);
        }
    }

    #[must_use]
    pub fn count_ones(&self) -> u32 {
        self.0.iter().map(|&w| (w as u64).count_ones()).sum()
    }

    pub fn encode(
        &self,
        write: &mut impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_26_3 {
            // 26.3 switched to `BitSet.toByteArray()`: little-endian bytes without trailing zeros
            let mut bytes: Vec<u8> = self.0.iter().flat_map(|w| w.to_le_bytes()).collect();
            while bytes.last() == Some(&0) {
                bytes.pop();
            }
            write.write_var_int(&bytes.len().try_into().map_err(|_| {
                WritingError::Message(format!("{} isn't representable as a VarInt", bytes.len()))
            })?)?;
            return write.write_slice(&bytes);
        }

        write.write_var_int(&self.0.len().try_into().map_err(|_| {
            WritingError::Message(format!("{} isn't representable as a VarInt", self.0.len()))
        })?)?;

        for b in &self.0 {
            write.write_i64_be(*b)?;
        }

        Ok(())
    }

    pub fn decode(
        read: &mut impl Read,
        version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        let length = read.get_var_int()?.0;
        if *version >= JavaMinecraftVersion::V_26_3 {
            let mut words = vec![0i64; (length.max(0) as usize).div_ceil(8)];
            for index in 0..length.max(0) as usize {
                words[index / 8] |= i64::from(read.get_u8()?) << (index % 8 * 8);
            }
            return Ok(Self(words.into_boxed_slice()));
        }

        let mut array: Vec<i64> = Vec::with_capacity(length.max(0) as usize);
        for _ in 0..length {
            array.push(read.get_i64_be()?);
        }
        Ok(Self(array.into_boxed_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::BitSet;
    use pumpkin_util::version::JavaMinecraftVersion;

    #[test]
    fn encodes_bytes_from_26_3() {
        let bits = BitSet::from_u64((1 << 17) | 1);
        let mut out = Vec::new();
        bits.encode(&mut out, &JavaMinecraftVersion::V_26_3)
            .unwrap();
        assert_eq!(out, [3, 0x01, 0x00, 0x02]);
        assert_eq!(
            BitSet::decode(&mut out.as_slice(), &JavaMinecraftVersion::V_26_3).unwrap(),
            bits
        );

        let mut empty = Vec::new();
        BitSet::from_u64(0)
            .encode(&mut empty, &JavaMinecraftVersion::V_26_3)
            .unwrap();
        assert_eq!(empty, [0]);
    }

    #[test]
    fn encodes_longs_before_26_3() {
        let mut out = Vec::new();
        BitSet::from_u64(1)
            .encode(&mut out, &JavaMinecraftVersion::V_26_2)
            .unwrap();
        assert_eq!(out, [1, 0, 0, 0, 0, 0, 0, 0, 1]);
    }
}

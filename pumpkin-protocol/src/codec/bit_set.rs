use std::io::Read;
use std::io::Write;

use serde::{Serialize, Serializer};

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
        // Read length
        let length = read.get_var_int()?;
        let mut array: Vec<i64> = Vec::with_capacity(length.0 as usize);
        for _ in 0..length.0 {
            let long = read.get_i64_be()?;
            array.push(long);
        }
        Ok(Self(array.into_boxed_slice()))
    }
}

impl Serialize for BitSet {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for &long in &self.0 {
            seq.serialize_element(&long)?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn bitset_encode_decode_empty() {
        let bs = BitSet(Box::new([]));
        let mut buf = Vec::new();
        bs.encode(&mut buf).unwrap();
        // VarInt(0) = [0x00]
        assert_eq!(buf, vec![0x00]);
        let decoded = BitSet::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded.0.len(), 0);
    }

    #[test]
    fn bitset_encode_decode_single() {
        let bs = BitSet(Box::new([0x0000_0000_0000_0001]));
        let mut buf = Vec::new();
        bs.encode(&mut buf).unwrap();
        // VarInt(1) = [0x01], then i64 big-endian
        assert_eq!(buf.len(), 1 + 8);
        assert_eq!(buf[0], 0x01);
        let decoded = BitSet::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded.0.as_ref(), &[1i64]);
    }

    #[test]
    fn bitset_encode_decode_multiple() {
        let bs = BitSet(Box::new([0xFF, 0x00, -1i64]));
        let mut buf = Vec::new();
        bs.encode(&mut buf).unwrap();
        let decoded = BitSet::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded.0.as_ref(), &[0xFF, 0x00, -1i64]);
    }

    #[test]
    fn bitset_serde_matches_encode() {
        use crate::ser::serializer::Serializer as NetSerializer;

        let bs = BitSet(Box::new([42, -100]));

        // Manual encode
        let mut manual_buf = Vec::new();
        bs.encode(&mut manual_buf).unwrap();

        // Serde serialize
        let mut serde_buf = Vec::new();
        let mut ser = NetSerializer::new(&mut serde_buf);
        bs.serialize(&mut ser).unwrap();

        assert_eq!(manual_buf, serde_buf, "Serde serialize should match manual encode");
    }
}

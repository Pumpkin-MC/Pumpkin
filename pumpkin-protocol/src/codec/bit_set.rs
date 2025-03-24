use std::io::Read;
use std::io::Write;
use std::num::NonZeroUsize;

use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};

use crate::ser::NetworkReadExt;
use crate::ser::NetworkWriteExt;
use crate::ser::ReadingError;
use crate::ser::WritingError;

use super::Codec;

pub struct BitSet(pub Box<[i64]>);

impl Codec<BitSet> for BitSet {
    /// The maximum size of the `BitSet` is `remaining / 8`.
    const MAX_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(usize::MAX) };

    fn written_size(&self) -> usize {
        let len = self.0.len();
        let len_varint = len as i32;

        // Calculate VarInt size using zig-zag encoding
        let zigzag = ((len_varint << 1) ^ (len_varint >> 31)) as u32;
        let mut varint_size = 0;
        let mut val = zigzag;
        loop {
            varint_size += 1;
            val >>= 7;
            if val == 0 {
                break;
            }
        }

        varint_size + 8 * len
    }

    fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        write.write_var_int(&self.0.len().into())?;
        for b in &self.0 {
            write.write_i64_be(*b)?;
        }

        Ok(())
    }

    fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        // Read length
        let length = read.get_var_int()?;
        let mut array: Vec<i64> = Vec::with_capacity(length.0 as usize);
        for _ in 0..length.0 {
            let long = read.get_i64_be()?;
            array.push(long);
        }
        Ok(BitSet(array.into_boxed_slice()))
    }
}

impl Serialize for BitSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for &value in self.0.iter() {
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for BitSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BitSetVisitor;

        impl<'de> Visitor<'de> for BitSetVisitor {
            type Value = BitSet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of i64 integers")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(value) = seq.next_element()? {
                    vec.push(value);
                }
                Ok(BitSet(vec.into_boxed_slice()))
            }
        }

        deserializer.deserialize_seq(BitSetVisitor)
    }
}
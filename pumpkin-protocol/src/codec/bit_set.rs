use std::io::Read;
use std::io::Write;

use serde::{Serialize, Serializer};

use crate::ser::NetworkReadExt;
use crate::ser::NetworkWriteExt;
use crate::ser::ReadingError;
use crate::ser::WritingError;

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
        Ok(BitSet(array.into_boxed_slice()))
    }
}

impl Serialize for BitSet {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

use core::str;
use std::io::{Read, Write};

use crate::{
    FixedBitSet,
    codec::{Codec, bit_set::BitSet, identifier::Identifier, var_int::VarInt, var_long::VarLong},
};

pub mod deserializer;
use thiserror::Error;
pub mod packet;
pub mod serializer;

#[derive(Debug, Error)]
pub enum ReadingError {
    #[error("EOF, Tried to read {0} but No bytes left to consume")]
    CleanEOF(String),
    #[error("incomplete: {0}")]
    Incomplete(String),
    #[error("too large: {0}")]
    TooLarge(String),
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Error)]
pub enum WritingError {
    #[error("IO error: {0}")]
    IoError(std::io::Error),
    #[error("Serde failure: {0}")]
    Serde(String),
    #[error("Failed to serialize packet: {0}")]
    Message(String),
}

pub trait NetworkReadExt {
    fn get_i8_be(&mut self) -> Result<i8, ReadingError>;
    fn get_u8_be(&mut self) -> Result<u8, ReadingError>;
    fn get_i16_be(&mut self) -> Result<i16, ReadingError>;
    fn get_u16_be(&mut self) -> Result<u16, ReadingError>;
    fn get_i32_be(&mut self) -> Result<i32, ReadingError>;
    fn get_u32_be(&mut self) -> Result<u32, ReadingError>;
    fn get_i64_be(&mut self) -> Result<i64, ReadingError>;
    fn get_u64_be(&mut self) -> Result<u64, ReadingError>;
    fn get_f32_be(&mut self) -> Result<f32, ReadingError>;
    fn get_f64_be(&mut self) -> Result<f64, ReadingError>;
    fn read_boxed_slice(&mut self, count: usize) -> Result<Box<[u8]>, ReadingError>;

    fn read_remaining_to_boxed_slice(&mut self, bound: usize) -> Result<Box<[u8]>, ReadingError>;

    fn get_bool(&mut self) -> Result<bool, ReadingError>;
    fn get_var_int(&mut self) -> Result<VarInt, ReadingError>;
    fn get_var_long(&mut self) -> Result<VarLong, ReadingError>;
    fn get_string_bounded(&mut self, bound: usize) -> Result<String, ReadingError>;
    fn get_string(&mut self) -> Result<String, ReadingError>;
    fn get_identifier(&mut self) -> Result<Identifier, ReadingError>;
    fn get_uuid(&mut self) -> Result<uuid::Uuid, ReadingError>;
    fn get_fixed_bitset(&mut self, bits: usize) -> Result<FixedBitSet, ReadingError>;

    fn get_option<G>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Option<G>, ReadingError>;

    fn get_list<G>(
        &mut self,
        parse: impl Fn(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Vec<G>, ReadingError>;
}

impl<R: Read> NetworkReadExt for R {
    //TODO: Macroize this
    fn get_i8_be(&mut self) -> Result<i8, ReadingError> {
        let mut buf = [0u8];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(i8::from_be_bytes(buf))
    }

    fn get_u8_be(&mut self) -> Result<u8, ReadingError> {
        let mut buf = [0u8];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(u8::from_be_bytes(buf))
    }

    fn get_i16_be(&mut self) -> Result<i16, ReadingError> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(i16::from_be_bytes(buf))
    }

    fn get_u16_be(&mut self) -> Result<u16, ReadingError> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(u16::from_be_bytes(buf))
    }

    fn get_i32_be(&mut self) -> Result<i32, ReadingError> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(i32::from_be_bytes(buf))
    }

    fn get_u32_be(&mut self) -> Result<u32, ReadingError> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(u32::from_be_bytes(buf))
    }

    fn get_i64_be(&mut self) -> Result<i64, ReadingError> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(i64::from_be_bytes(buf))
    }

    fn get_u64_be(&mut self) -> Result<u64, ReadingError> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(u64::from_be_bytes(buf))
    }
    fn get_f32_be(&mut self) -> Result<f32, ReadingError> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(f32::from_be_bytes(buf))
    }

    fn get_f64_be(&mut self) -> Result<f64, ReadingError> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(f64::from_be_bytes(buf))
    }

    fn read_boxed_slice(&mut self, count: usize) -> Result<Box<[u8]>, ReadingError> {
        let mut buf = vec![0u8; count];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(buf.into())
    }

    fn read_remaining_to_boxed_slice(&mut self, bound: usize) -> Result<Box<[u8]>, ReadingError> {
        let mut return_buf = Vec::new();

        // TODO: We can probably remove the temp buffer somehow
        let mut temp_buf = [0; 1024];
        loop {
            let bytes_read = self
                .read(&mut temp_buf)
                .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

            if bytes_read == 0 {
                break;
            }

            if return_buf.len() + bytes_read > bound {
                return Err(ReadingError::TooLarge(
                    "Read remaining too long".to_string(),
                ));
            }

            return_buf.extend(&temp_buf[..bytes_read]);
        }
        Ok(return_buf.into_boxed_slice())
    }

    fn get_bool(&mut self) -> Result<bool, ReadingError> {
        let byte = self.get_u8_be()?;
        Ok(byte != 0)
    }

    fn get_var_int(&mut self) -> Result<VarInt, ReadingError> {
        VarInt::decode(self)
    }

    fn get_var_long(&mut self) -> Result<VarLong, ReadingError> {
        VarLong::decode(self)
    }

    fn get_string_bounded(&mut self, bound: usize) -> Result<String, ReadingError> {
        let size = self.get_var_int()?.0 as usize;
        if size > bound {
            return Err(ReadingError::TooLarge("string".to_string()));
        }

        let data = self.read_boxed_slice(size)?;
        String::from_utf8(data.into()).map_err(|e| ReadingError::Message(e.to_string()))
    }

    fn get_string(&mut self) -> Result<String, ReadingError> {
        self.get_string_bounded(i16::MAX as usize)
    }

    fn get_identifier(&mut self) -> Result<Identifier, ReadingError> {
        Identifier::decode(self)
    }

    fn get_uuid(&mut self) -> Result<uuid::Uuid, ReadingError> {
        let mut bytes = [0u8; 16];
        self.read_exact(&mut bytes)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;
        Ok(uuid::Uuid::from_slice(&bytes).expect("Failed to parse UUID"))
    }

    fn get_fixed_bitset(&mut self, bits: usize) -> Result<FixedBitSet, ReadingError> {
        let bytes = self.read_boxed_slice(bits.div_ceil(8))?;
        Ok(bytes)
    }

    fn get_option<G>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Option<G>, ReadingError> {
        if self.get_bool()? {
            Ok(Some(parse(self)?))
        } else {
            Ok(None)
        }
    }

    fn get_list<G>(
        &mut self,
        parse: impl Fn(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Vec<G>, ReadingError> {
        let len = self.get_var_int()?.0 as usize;
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(parse(self)?);
        }
        Ok(list)
    }
}

pub trait NetworkWriteExt {
    fn write_i8_be(&mut self, data: i8) -> Result<(), WritingError>;
    fn write_u8_be(&mut self, data: u8) -> Result<(), WritingError>;
    fn write_i16_be(&mut self, data: i16) -> Result<(), WritingError>;
    fn write_u16_be(&mut self, data: u16) -> Result<(), WritingError>;
    fn write_i32_be(&mut self, data: i32) -> Result<(), WritingError>;
    fn write_u32_be(&mut self, data: u32) -> Result<(), WritingError>;
    fn write_i64_be(&mut self, data: i64) -> Result<(), WritingError>;
    fn write_u64_be(&mut self, data: u64) -> Result<(), WritingError>;
    fn write_f32_be(&mut self, data: f32) -> Result<(), WritingError>;
    fn write_f64_be(&mut self, data: f64) -> Result<(), WritingError>;
    fn write_slice(&mut self, data: &[u8]) -> Result<(), WritingError>;

    fn write_bool(&mut self, data: bool) -> Result<(), WritingError>;
    fn write_var_int(&mut self, data: &VarInt) -> Result<(), WritingError>;
    fn write_var_long(&mut self, data: &VarLong) -> Result<(), WritingError>;
    fn write_string_bounded(&mut self, data: &str, bound: usize) -> Result<(), WritingError>;
    fn write_string(&mut self, data: &str) -> Result<(), WritingError>;
    fn write_identifier(&mut self, data: &Identifier) -> Result<(), WritingError>;
    fn write_uuid(&mut self, data: &uuid::Uuid) -> Result<(), WritingError>;
    fn write_bitset(&mut self, bitset: &BitSet) -> Result<(), WritingError>;

    fn write_option<G>(
        &mut self,
        data: &Option<G>,
        write: impl FnOnce(&mut Self, &G) -> Result<(), WritingError>,
    ) -> Result<(), WritingError>;

    fn write_list<G>(
        &mut self,
        data: &[G],
        write: impl Fn(&mut Self, &G) -> Result<(), WritingError>,
    ) -> Result<(), WritingError>;
}

impl<W: Write> NetworkWriteExt for W {
    fn write_i8_be(&mut self, data: i8) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_u8_be(&mut self, data: u8) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_i16_be(&mut self, data: i16) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_u16_be(&mut self, data: u16) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_i32_be(&mut self, data: i32) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_u32_be(&mut self, data: u32) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_i64_be(&mut self, data: i64) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_u64_be(&mut self, data: u64) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_f32_be(&mut self, data: f32) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_f64_be(&mut self, data: f64) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_slice(&mut self, data: &[u8]) -> Result<(), WritingError> {
        self.write_all(data).map_err(WritingError::IoError)
    }

    fn write_bool(&mut self, data: bool) -> Result<(), WritingError> {
        if data {
            self.write_u8_be(1)
        } else {
            self.write_u8_be(0)
        }
    }

    fn write_var_int(&mut self, data: &VarInt) -> Result<(), WritingError> {
        data.encode(self)
    }

    fn write_var_long(&mut self, data: &VarLong) -> Result<(), WritingError> {
        data.encode(self)
    }

    fn write_string_bounded(&mut self, data: &str, bound: usize) -> Result<(), WritingError> {
        assert!(data.len() <= bound);
        self.write_var_int(&data.len().into())?;
        self.write_all(data.as_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_string(&mut self, data: &str) -> Result<(), WritingError> {
        self.write_string_bounded(data, i16::MAX as usize)
    }

    fn write_identifier(&mut self, data: &Identifier) -> Result<(), WritingError> {
        data.encode(self)
    }

    fn write_uuid(&mut self, data: &uuid::Uuid) -> Result<(), WritingError> {
        let (first, second) = data.as_u64_pair();
        self.write_u64_be(first)?;
        self.write_u64_be(second)
    }

    fn write_bitset(&mut self, data: &BitSet) -> Result<(), WritingError> {
        data.encode(self)
    }

    fn write_option<G>(
        &mut self,
        data: &Option<G>,
        writer: impl FnOnce(&mut Self, &G) -> Result<(), WritingError>,
    ) -> Result<(), WritingError> {
        if let Some(data) = data {
            self.write_bool(true)?;
            writer(self, data)
        } else {
            self.write_bool(false)
        }
    }

    fn write_list<G>(
        &mut self,
        list: &[G],
        writer: impl Fn(&mut Self, &G) -> Result<(), WritingError>,
    ) -> Result<(), WritingError> {
        self.write_var_int(&list.len().into())?;
        for data in list {
            writer(self, data)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use serde::{Deserialize, Serialize};

    use crate::{
        VarInt,
        ser::{deserializer, serializer},
    };

    #[test]
    fn test_i32_reserialize() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
        struct Foo {
            bar: i32,
        }
        let foo = Foo { bar: 69 };
        let mut bytes = Vec::new();
        let mut serializer = serializer::Serializer::new(&mut bytes);
        foo.serialize(&mut serializer).unwrap();

        let cursor = Cursor::new(bytes);
        let deserialized: Foo =
            Foo::deserialize(&mut deserializer::Deserializer::new(cursor)).unwrap();

        assert_eq!(foo, deserialized);
    }

    #[test]
    fn test_varint_reserialize() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
        struct Foo {
            bar: VarInt,
        }
        let foo = Foo { bar: 69.into() };
        let mut bytes = Vec::new();
        let mut serializer = serializer::Serializer::new(&mut bytes);
        foo.serialize(&mut serializer).unwrap();

        let cursor = Cursor::new(bytes);
        let deserialized: Foo =
            Foo::deserialize(&mut deserializer::Deserializer::new(cursor)).unwrap();

        assert_eq!(foo, deserialized);
    }
}

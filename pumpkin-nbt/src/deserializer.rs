use crate::*;
use io::{ErrorKind, Read};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct ReadAdaptor<R: Read> {
    reader: R,
}

impl<R: Read> ReadAdaptor<R> {
    pub fn is_eof(&mut self) -> Result<bool> {
        let mut buf = [];
        match self.reader.read_exact(&mut buf) {
            Ok(()) => Ok(false),
            Err(e) => match e.kind() {
                ErrorKind::UnexpectedEof => Ok(true),
                _ => Err(Error::Incomplete(e)),
            },
        }
    }

    //TODO: Macroize this
    pub fn get_u8_be(&mut self) -> Result<u8> {
        let mut buf = [0u8];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(u8::from_be_bytes(buf))
    }

    pub fn get_i8_be(&mut self) -> Result<i8> {
        let mut buf = [0u8];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(i8::from_be_bytes(buf))
    }

    pub fn get_i16_be(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(i16::from_be_bytes(buf))
    }

    pub fn get_u16_be(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(u16::from_be_bytes(buf))
    }

    pub fn get_u32_be(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(u32::from_be_bytes(buf))
    }

    pub fn get_i32_be(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(i32::from_be_bytes(buf))
    }

    pub fn get_i64_be(&mut self) -> Result<i64> {
        let mut buf = [0u8; 8];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(i64::from_be_bytes(buf))
    }

    pub fn get_f32_be(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(f32::from_be_bytes(buf))
    }

    pub fn get_f64_be(&mut self) -> Result<f64> {
        let mut buf = [0u8; 8];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(f64::from_be_bytes(buf))
    }

    pub fn read_to_bytes(&mut self, count: usize) -> Result<Bytes> {
        let mut buf = vec![0u8; count];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(buf.into())
    }
}

#[derive(Debug)]
pub struct Deserializer<R: Read> {
    input: ReadAdaptor<R>,
    tag_to_deserialize: Option<u8>,
    is_named: bool,
}

impl<R: Read> Deserializer<R> {
    pub fn new(input: R, is_named: bool) -> Self {
        Deserializer {
            input: ReadAdaptor { reader: input },
            tag_to_deserialize: None,
            is_named,
        }
    }
}

/// Deserializes struct using Serde Deserializer from normal NBT
pub fn from_bytes<'a, T, R>(r: R) -> Result<T>
where
    T: Deserialize<'a>,
    R: Read,
{
    let mut deserializer = Deserializer::new(r, true);
    T::deserialize(&mut deserializer)
}

/// Deserializes struct using Serde Deserializer from normal NBT
pub fn from_bytes_unnamed<'a, T, R>(r: R) -> Result<T>
where
    T: Deserialize<'a>,
    R: Read,
{
    let mut deserializer = Deserializer::new(r, false);
    T::deserialize(&mut deserializer)
}

impl<'de, R: Read> de::Deserializer<'de> for &mut Deserializer<R> {
    type Error = Error;

    forward_to_deserialize_any!(i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 seq char str string bytes byte_buf tuple tuple_struct enum ignored_any unit unit_struct option newtype_struct);

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let tag_to_deserialize = self.tag_to_deserialize.unwrap();

        let list_type = match tag_to_deserialize {
            LIST_ID => Some(self.input.get_u8_be()?),
            INT_ARRAY_ID => Some(INT_ID),
            LONG_ARRAY_ID => Some(LONG_ID),
            BYTE_ARRAY_ID => Some(BYTE_ID),
            _ => None,
        };

        if let Some(list_type) = list_type {
            let remaining_values = self.input.get_u32_be()?;
            return visitor.visit_seq(ListAccess {
                de: self,
                list_type,
                remaining_values,
            });
        }

        let result: Result<V::Value> = Ok(
            match NbtTag::deserialize_data(&mut self.input, tag_to_deserialize)? {
                NbtTag::Byte(value) => visitor.visit_i8::<Error>(value)?,
                NbtTag::Short(value) => visitor.visit_i16::<Error>(value)?,
                NbtTag::Int(value) => visitor.visit_i32::<Error>(value)?,
                NbtTag::Long(value) => visitor.visit_i64::<Error>(value)?,
                NbtTag::Float(value) => visitor.visit_f32::<Error>(value)?,
                NbtTag::Double(value) => visitor.visit_f64::<Error>(value)?,
                NbtTag::String(value) => visitor.visit_string::<Error>(value)?,
                _ => unreachable!(),
            },
        );
        self.tag_to_deserialize = None;
        result
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.tag_to_deserialize.unwrap() == BYTE_ID {
            let value = self.input.get_u8_be()?;
            if value != 0 {
                return visitor.visit_bool(true);
            }
        }
        visitor.visit_bool(false)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.tag_to_deserialize.is_none() {
            let next_byte = self.input.get_u8_be()?;
            if next_byte != COMPOUND_ID {
                return Err(Error::NoRootCompound(next_byte));
            }

            if self.is_named {
                // Consume struct name
                NbtTag::deserialize(&mut self.input)?;
            }
        }

        let value = visitor.visit_map(CompoundAccess { de: self })?;
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let str = get_nbt_string(&mut self.input).map_err(|_| Error::Cesu8DecodingError)?;
        visitor.visit_string(str)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

struct CompoundAccess<'a, R: Read> {
    de: &'a mut Deserializer<R>,
}

impl<'de, R: Read> MapAccess<'de> for CompoundAccess<'_, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        let tag = self.de.input.get_u8_be()?;
        self.de.tag_to_deserialize = Some(tag);

        if tag == END_ID {
            return Ok(None);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct ListAccess<'a, R: Read> {
    de: &'a mut Deserializer<R>,
    remaining_values: u32,
    list_type: u8,
}

impl<'de, R: Read> SeqAccess<'de> for ListAccess<'_, R> {
    type Error = Error;

    fn next_element_seed<E>(&mut self, seed: E) -> Result<Option<E::Value>>
    where
        E: DeserializeSeed<'de>,
    {
        if self.remaining_values == 0 {
            return Ok(None);
        }

        self.remaining_values -= 1;
        self.de.tag_to_deserialize = Some(self.list_type);
        seed.deserialize(&mut *self.de).map(Some)
    }
}

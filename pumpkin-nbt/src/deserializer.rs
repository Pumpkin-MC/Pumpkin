use crate::*;
use io::Read;
use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct ReadAdaptor<R: Read> {
    reader: R,
}

impl<R: Read> ReadAdaptor<R> {
    pub fn new(r: R) -> Self {
        Self { reader: r }
    }
}

impl<R: Read> ReadAdaptor<R> {
    pub fn skip_bytes(&mut self, count: u64) -> Result<()> {
        let _ = io::copy(&mut self.reader.by_ref().take(count), &mut io::sink())
            .map_err(Error::Incomplete)?;
        Ok(())
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

    pub fn read_boxed_slice(&mut self, count: usize) -> Result<Box<[u8]>> {
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
    tag_to_deserialize_stack: Vec<u8>,
    // Yes, this breaks with recursion. Just an attempt at a sanity check
    in_list: bool,
    is_named: bool,
}

impl<R: Read> Deserializer<R> {
    pub fn new(input: R, is_named: bool) -> Self {
        Deserializer {
            input: ReadAdaptor { reader: input },
            tag_to_deserialize_stack: Vec::new(),
            in_list: false,
            is_named,
        }
    }
}

/// Deserializes struct using Serde Deserializer from normal NBT
pub fn from_bytes<'a, T>(r: impl Read) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::new(r, true);
    T::deserialize(&mut deserializer)
}

/// Deserializes struct using Serde Deserializer from network NBT
pub fn from_bytes_unnamed<'a, T>(r: impl Read) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::new(r, false);
    T::deserialize(&mut deserializer)
}

impl<'de, R: Read> de::Deserializer<'de> for &mut Deserializer<R> {
    type Error = Error;

    forward_to_deserialize_any! {
        i8 i16 i32 i64 f32 f64 char str string unit unit_struct seq tuple tuple_struct
        bytes newtype_struct byte_buf
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let Some(tag) = self.tag_to_deserialize_stack.pop() else {
            return Err(Error::SerdeError("Ignoring nothing!".to_string()));
        };

        NbtTag::skip_data(&mut self.input, tag)?;
        visitor.visit_unit()
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let Some(tag_to_deserialize) = self.tag_to_deserialize_stack.pop() else {
            return Err(Error::SerdeError(
                "The top level must be a component (e.g. a struct)".to_string(),
            ));
        };

        match tag_to_deserialize {
            END_ID => Err(Error::SerdeError(
                "Trying to deserialize an END tag!".to_string(),
            )),
            LIST_ID | INT_ARRAY_ID | LONG_ARRAY_ID | BYTE_ARRAY_ID => {
                let list_type = match tag_to_deserialize {
                    LIST_ID => self.input.get_u8_be()?,
                    INT_ARRAY_ID => INT_ID,
                    LONG_ARRAY_ID => LONG_ID,
                    BYTE_ARRAY_ID => BYTE_ID,
                    _ => unreachable!(),
                };

                let remaining_values = self.input.get_i32_be()?;
                if remaining_values < 0 {
                    return Err(Error::NegativeLength(remaining_values));
                }

                let result = visitor.visit_seq(ListAccess {
                    de: self,
                    list_type,
                    remaining_values: remaining_values as usize,
                })?;
                Ok(result)
            }
            COMPOUND_ID => visitor.visit_map(CompoundAccess { de: self }),
            _ => {
                let result = match NbtTag::deserialize_data(&mut self.input, tag_to_deserialize)? {
                    NbtTag::Byte(value) => visitor.visit_i8::<Error>(value)?,
                    NbtTag::Short(value) => visitor.visit_i16::<Error>(value)?,
                    NbtTag::Int(value) => visitor.visit_i32::<Error>(value)?,
                    NbtTag::Long(value) => visitor.visit_i64::<Error>(value)?,
                    NbtTag::Float(value) => visitor.visit_f32::<Error>(value)?,
                    NbtTag::Double(value) => visitor.visit_f64::<Error>(value)?,
                    NbtTag::String(value) => visitor.visit_string::<Error>(value)?,
                    _ => unreachable!(),
                };
                Ok(result)
            }
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.in_list {
            let value = self.input.get_u8_be()?;
            visitor.visit_u8::<Error>(value)
        } else {
            Err(Error::UnsupportedType(
                "u8; NBT only supports signed values".to_string(),
            ))
        }
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType(
            "u16; NBT only supports signed values".to_string(),
        ))
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType(
            "u32; NBT only supports signed values".to_string(),
        ))
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType(
            "u64; NBT only supports signed values".to_string(),
        ))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Some(tag_id) = self.tag_to_deserialize_stack.last() {
            if *tag_id == BYTE_ID {
                let value = self.input.get_u8_be()?;
                if value != 0 {
                    visitor.visit_bool(true)
                } else {
                    visitor.visit_bool(false)
                }
            } else {
                Err(Error::UnsupportedType(format!(
                    "Non-byte bool (found type {})",
                    tag_id
                )))
            }
        } else {
            Err(Error::SerdeError(
                "Wanted to deserialize a bool, but there was no type hint on the stack!"
                    .to_string(),
            ))
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let variant = get_nbt_string(&mut self.input)?;
        visitor.visit_enum(variant.into_deserializer())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // None is not encoded, so no need for it
        visitor.visit_some(self)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Some(tag_id) = self.tag_to_deserialize_stack.pop() {
            if tag_id != COMPOUND_ID {
                return Err(Error::SerdeError(
                    "Trying to deserialize a map without a compound ID".to_string(),
                ));
            }
        } else {
            let next_byte = self.input.get_u8_be()?;
            if next_byte != COMPOUND_ID {
                return Err(Error::NoRootCompound(next_byte));
            }

            if self.is_named {
                // Consume struct name
                let _ = get_nbt_string(&mut self.input)?;
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
        let str = get_nbt_string(&mut self.input)?;
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
        self.de.tag_to_deserialize_stack.push(tag);

        if tag == END_ID {
            return Ok(None);
        }

        seed.deserialize(MapKey { de: self.de }).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct MapKey<'a, R: Read> {
    de: &'a mut Deserializer<R>,
}

impl<'de, R: Read> de::Deserializer<'de> for MapKey<'_, R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let key = get_nbt_string(&mut self.de.input)?;
        visitor.visit_string(key)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit unit_struct seq tuple tuple_struct map
        struct identifier ignored_any bytes enum newtype_struct byte_buf option
    }
}

struct ListAccess<'a, R: Read> {
    de: &'a mut Deserializer<R>,
    remaining_values: usize,
    list_type: u8,
}

impl<'de, R: Read> SeqAccess<'de> for ListAccess<'_, R> {
    type Error = Error;

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining_values)
    }

    fn next_element_seed<E>(&mut self, seed: E) -> Result<Option<E::Value>>
    where
        E: DeserializeSeed<'de>,
    {
        if self.remaining_values == 0 {
            return Ok(None);
        }

        self.remaining_values -= 1;
        self.de.tag_to_deserialize_stack.push(self.list_type);
        self.de.in_list = true;
        let result = seed.deserialize(&mut *self.de).map(Some);
        self.de.in_list = false;

        result
    }
}

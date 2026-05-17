use std::cell::RefCell;
use std::io::{Seek, SeekFrom};

use crate::{
    BYTE_ARRAY_ID, BYTE_ID, COMPOUND_ID, END_ID, Error, INT_ARRAY_ID, INT_ID, LIST_ID,
    LONG_ARRAY_ID, LONG_ID, NbtTag, io,
};
use io::Read;
use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, forward_to_deserialize_any};

pub type Result<T> = std::result::Result<T, Error>;

thread_local! {
    pub static CURR_VISITOR_LIST_TYPE: RefCell<Option<u8>> = const { std::cell::RefCell::new(None) };
}

pub(super) fn take_curr_visitor_seq_list_id() -> Option<u8> {
    CURR_VISITOR_LIST_TYPE.with(std::cell::RefCell::take)
}

pub(super) fn set_curr_visitor_seq_list_id(tag: Option<u8>) {
    CURR_VISITOR_LIST_TYPE.with(|cell| {
        *cell.borrow_mut() = tag;
    });
}

pub trait NbtReadHelper {
    type Reader: Read + Seek;

    fn reader(&mut self) -> &mut Self::Reader;

    fn skip_bytes(&mut self, count: i64) -> Result<()> {
        self.reader()
            .seek(SeekFrom::Current(count))
            .map_err(Error::Incomplete)?;
        Ok(())
    }
    fn skip_u8(&mut self) -> Result<()> {
        self.skip_bytes(1)
    }
    fn skip_i8(&mut self) -> Result<()> {
        self.skip_bytes(1)
    }
    fn skip_i16(&mut self) -> Result<()> {
        self.skip_bytes(2)
    }
    fn skip_i32(&mut self) -> Result<()> {
        self.skip_bytes(4)
    }
    fn skip_i64(&mut self) -> Result<()> {
        self.skip_bytes(8)
    }
    fn skip_f32(&mut self) -> Result<()> {
        self.skip_bytes(4)
    }
    fn skip_f64(&mut self) -> Result<()> {
        self.skip_bytes(8)
    }
    fn skip_string(&mut self) -> Result<()>;

    fn get_u8(&mut self) -> Result<u8>;
    fn get_i8(&mut self) -> Result<i8>;
    fn get_i16(&mut self) -> Result<i16>;
    fn get_i32(&mut self) -> Result<i32>;
    fn get_i64(&mut self) -> Result<i64>;
    fn get_f32(&mut self) -> Result<f32>;
    fn get_f64(&mut self) -> Result<f64>;
    fn get_string(&mut self) -> Result<String>;
}

macro_rules! define_get_number_be {
    ($name:ident, $type:ty) => {
        fn $name(&mut self) -> Result<$type> {
            let mut buf = [0u8; std::mem::size_of::<$type>()];
            self.reader
                .read_exact(&mut buf)
                .map_err(Error::Incomplete)?;

            Ok(<$type>::from_be_bytes(buf))
        }
    };
}

macro_rules! define_get_number_le {
    ($name:ident, $type:ty) => {
        fn $name(&mut self) -> Result<$type> {
            let mut buf = [0u8; std::mem::size_of::<$type>()];
            self.reader
                .read_exact(&mut buf)
                .map_err(Error::Incomplete)?;

            Ok(<$type>::from_le_bytes(buf))
        }
    };
}

pub struct NbtReadHelperJava<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> NbtReadHelperJava<R> {
    pub const fn new(r: R) -> Self {
        Self { reader: r }
    }
}

pub struct NbtReadHelperBedrock<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> NbtReadHelperBedrock<R> {
    pub const fn new(r: R) -> Self {
        Self { reader: r }
    }
}

impl<R: Read + Seek> NbtReadHelperJava<R> {
    define_get_number_be!(get_string_len, u16);
}

impl<R: Read + Seek> NbtReadHelper for NbtReadHelperJava<R> {
    type Reader = R;

    fn reader(&mut self) -> &mut R {
        &mut self.reader
    }

    fn skip_string(&mut self) -> Result<()> {
        let len = self.get_string_len()? as i64;
        self.skip_bytes(len)
    }

    define_get_number_be!(get_u8, u8);
    define_get_number_be!(get_i8, i8);
    define_get_number_be!(get_i16, i16);
    define_get_number_be!(get_i32, i32);
    define_get_number_be!(get_i64, i64);
    define_get_number_be!(get_f32, f32);
    define_get_number_be!(get_f64, f64);

    fn get_string(&mut self) -> Result<String> {
        let len = self.get_string_len()? as usize;

        let mut buf = vec![0u8; len];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        let string = cesu8::from_java_cesu8(&buf).map_err(|_| Error::Cesu8DecodingError)?;

        Ok(string.into_owned())
    }
}

impl<R: Read + Seek> NbtReadHelperBedrock<R> {
    fn get_var_u32(&mut self) -> Result<u32> {
        // LEB128
        let mut val = 0;
        for i in 0..5 {
            let byte = self.get_u8()?;
            val |= (u32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(val);
            }
        }
        Err(Error::VarIntTooLarge)
    }

    fn get_var_i32(&mut self) -> Result<i32> {
        let val = self.get_var_u32()?;
        // ZigZag
        Ok(((val >> 1) as i32) ^ -((val as i32) & 1))
    }

    fn get_var_u64(&mut self) -> Result<u64> {
        // LEB128
        let mut val = 0;
        for i in 0..10 {
            let byte = self.get_u8()?;
            val |= (u64::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(val);
            }
        }
        Err(Error::VarLongTooLarge)
    }

    fn get_var_i64(&mut self) -> Result<i64> {
        let val = self.get_var_u64()?;
        // ZigZag
        Ok(((val >> 1) as i64) ^ -((val as i64) & 1))
    }

    fn get_string_len(&mut self) -> Result<u32> {
        self.get_var_u32()
    }
}

impl<R: Read + Seek> NbtReadHelper for NbtReadHelperBedrock<R> {
    type Reader = R;

    fn reader(&mut self) -> &mut Self::Reader {
        &mut self.reader
    }

    fn skip_i32(&mut self) -> Result<()> {
        for _ in 0..5 {
            if self.get_u8()? & 0x80 == 0 {
                return Ok(());
            }
        }
        Err(Error::VarIntTooLarge)
    }
    fn skip_i64(&mut self) -> Result<()> {
        for _ in 0..10 {
            if self.get_u8()? & 0x80 == 0 {
                return Ok(());
            }
        }
        Err(Error::VarLongTooLarge)
    }
    fn skip_string(&mut self) -> Result<()> {
        let len = self.get_string_len()? as i64;
        self.skip_bytes(len)
    }

    define_get_number_le!(get_u8, u8);
    define_get_number_le!(get_i8, i8);
    define_get_number_le!(get_i16, i16);
    fn get_i32(&mut self) -> Result<i32> {
        self.get_var_i32()
    }
    fn get_i64(&mut self) -> Result<i64> {
        self.get_var_i64()
    }
    define_get_number_le!(get_f32, f32);
    define_get_number_le!(get_f64, f64);

    fn get_string(&mut self) -> Result<String> {
        let len = self.get_string_len()? as usize;

        let mut buf = vec![0u8; len];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        String::from_utf8(buf).map_err(|_| Error::Utf8DecodingError)
    }
}

pub struct Deserializer<R: NbtReadHelper> {
    input: R,
    tag_to_deserialize_stack: Option<u8>,
    // Yes, this breaks with recursion. Just an attempt at a sanity check
    in_list: bool,
    is_named: bool,
}

impl<R: NbtReadHelper> Deserializer<R> {
    pub const fn new(input: R, is_named: bool) -> Self {
        Self {
            input,
            tag_to_deserialize_stack: None,
            in_list: false,
            is_named,
        }
    }
}

/// Deserializes struct using Serde Deserializer from normal NBT
pub fn from_bytes<'a, T: Deserialize<'a>>(r: impl Read + Seek) -> Result<T> {
    let mut deserializer = Deserializer::new(NbtReadHelperJava::new(r), true);
    T::deserialize(&mut deserializer)
}

/// Deserializes struct using Serde Deserializer from network NBT
pub fn from_bytes_unnamed<'a, T: Deserialize<'a>>(r: impl Read + Seek) -> Result<T> {
    let mut deserializer = Deserializer::new(NbtReadHelperJava::new(r), false);
    T::deserialize(&mut deserializer)
}

/// Deserializes struct using Serde Deserializer from Bedrock network NBT
pub fn from_bytes_bedrock<'a, T: Deserialize<'a>>(r: impl Read + Seek) -> Result<T> {
    let mut deserializer = Deserializer::new(NbtReadHelperBedrock::new(r), true);
    T::deserialize(&mut deserializer)
}

impl<'de, R: NbtReadHelper> de::Deserializer<'de> for &mut Deserializer<R> {
    type Error = Error;

    forward_to_deserialize_any! {
        i8 i16 i32 i64 f32 f64 char str string unit unit_struct seq tuple tuple_struct
        bytes newtype_struct byte_buf
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let Some(tag) = self.tag_to_deserialize_stack else {
            return Err(Error::SerdeError("Ignoring nothing!".to_string()));
        };

        NbtTag::skip_data(&mut self.input, tag)?;

        visitor.visit_unit()
    }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let Some(tag_to_deserialize) = self.tag_to_deserialize_stack else {
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
                    LIST_ID => self.input.get_u8()?,
                    INT_ARRAY_ID => INT_ID,
                    LONG_ARRAY_ID => LONG_ID,
                    BYTE_ARRAY_ID => BYTE_ID,
                    _ => unreachable!(),
                };

                let remaining_values = self.input.get_i32()?;
                if remaining_values < 0 {
                    return Err(Error::NegativeLength(remaining_values));
                }

                //TODO this is a bit hacky but I couldn't think of a better way
                // This flag gets auto cleared in visit_seq
                set_curr_visitor_seq_list_id(Some(list_type));
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

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.in_list {
            let value = self.input.get_u8()?;
            visitor.visit_u8::<Error>(value)
        } else {
            Err(Error::UnsupportedType(
                "u8; NBT only supports signed values".to_string(),
            ))
        }
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.input.get_i16()?;
        visitor.visit_i16::<Error>(value)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // ZigZag might make this a problem for Bedrock...
        let value = self.input.get_i32()?;
        visitor.visit_i32::<Error>(value)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // ZigZag might make this a problem for Bedrock...
        let value = self.input.get_i64()?;
        visitor.visit_i64::<Error>(value)
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.tag_to_deserialize_stack.unwrap() == BYTE_ID {
            let value = self.input.get_u8()?;
            if value != 0 {
                return visitor.visit_bool(true);
            }
        }
        visitor.visit_bool(false)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let variant = self.input.get_string()?;
        visitor.visit_enum(variant.into_deserializer())
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // None is not encoded, so no need for it
        visitor.visit_some(self)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(tag_id) = self.tag_to_deserialize_stack {
            if tag_id != COMPOUND_ID {
                return Err(Error::SerdeError(format!(
                    "Trying to deserialize a map without a compound ID (id {tag_id})"
                )));
            }
        } else {
            let next_byte = self.input.get_u8()?;
            if next_byte != COMPOUND_ID {
                return Err(Error::NoRootCompound(next_byte));
            }

            if self.is_named {
                self.input.skip_string()?;
            }
        }

        let value = visitor.visit_map(CompoundAccess { de: self })?;
        Ok(value)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_map(visitor)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let str = self.input.get_string()?;
        visitor.visit_string(str)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

struct CompoundAccess<'a, R: NbtReadHelper> {
    de: &'a mut Deserializer<R>,
}

impl<'de, R: NbtReadHelper> MapAccess<'de> for CompoundAccess<'_, R> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        let tag = self.de.input.get_u8()?;
        self.de.tag_to_deserialize_stack = Some(tag);

        if tag == END_ID {
            return Ok(None);
        }

        seed.deserialize(MapKey { de: self.de }).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        seed.deserialize(&mut *self.de)
    }
}

struct MapKey<'a, R: NbtReadHelper> {
    de: &'a mut Deserializer<R>,
}

impl<'de, R: NbtReadHelper> de::Deserializer<'de> for MapKey<'_, R> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let key = self.de.input.get_string()?;
        visitor.visit_string(key)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit unit_struct seq tuple tuple_struct map
        struct identifier ignored_any bytes enum newtype_struct byte_buf option
    }
}

struct ListAccess<'a, R: NbtReadHelper> {
    de: &'a mut Deserializer<R>,
    remaining_values: usize,
    list_type: u8,
}

impl<'de, R: NbtReadHelper> SeqAccess<'de> for ListAccess<'_, R> {
    type Error = Error;

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining_values)
    }

    fn next_element_seed<E: DeserializeSeed<'de>>(&mut self, seed: E) -> Result<Option<E::Value>> {
        if self.remaining_values == 0 {
            return Ok(None);
        }

        self.remaining_values -= 1;
        self.de.tag_to_deserialize_stack = Some(self.list_type);
        self.de.in_list = true;
        let result = seed.deserialize(&mut *self.de).map(Some);
        self.de.in_list = false;

        result
    }
}

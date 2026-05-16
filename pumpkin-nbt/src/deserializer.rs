use std::cell::RefCell;
use std::io::{Seek, SeekFrom};

use crate::{
    BYTE_ARRAY_ID, BYTE_ID, COMPOUND_ID, END_ID, Error, INT_ARRAY_ID, INT_ID, LIST_ID,
    LONG_ARRAY_ID, LONG_ID, NbtTag, get_nbt_string, get_nbt_string_bedrock, io,
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

pub struct NbtReadHelper<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> NbtReadHelper<R> {
    pub const fn new(r: R) -> Self {
        Self { reader: r }
    }
}

macro_rules! define_get_number_be {
    ($name:ident, $type:ty) => {
        pub fn $name(&mut self) -> Result<$type> {
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
        pub fn $name(&mut self) -> Result<$type> {
            let mut buf = [0u8; std::mem::size_of::<$type>()];
            self.reader
                .read_exact(&mut buf)
                .map_err(Error::Incomplete)?;

            Ok(<$type>::from_le_bytes(buf))
        }
    };
}

impl<R: Read + Seek> NbtReadHelper<R> {
    pub fn stream_position(&mut self) -> Result<u64> {
        self.reader.stream_position().map_err(Error::Incomplete)
    }

    pub fn skip_bytes(&mut self, count: i64) -> Result<()> {
        self.reader
            .seek(SeekFrom::Current(count))
            .map_err(Error::Incomplete)?;
        Ok(())
    }

    define_get_number_be!(get_u8_be, u8);
    define_get_number_be!(get_i8_be, i8);
    define_get_number_be!(get_u16_be, u16);
    define_get_number_be!(get_i16_be, i16);
    define_get_number_be!(get_u32_be, u32);
    define_get_number_be!(get_i32_be, i32);
    define_get_number_be!(get_u64_be, u64);
    define_get_number_be!(get_i64_be, i64);
    define_get_number_be!(get_f32_be, f32);
    define_get_number_be!(get_f64_be, f64);

    define_get_number_le!(get_u8_le, u8);
    define_get_number_le!(get_i8_le, i8);
    define_get_number_le!(get_u16_le, u16);
    define_get_number_le!(get_i16_le, i16);
    define_get_number_le!(get_u32_le, u32);
    define_get_number_le!(get_i32_le, i32);
    define_get_number_le!(get_u64_le, u64);
    define_get_number_le!(get_i64_le, i64);
    define_get_number_le!(get_f32_le, f32);
    define_get_number_le!(get_f64_le, f64);

    pub fn get_var_u32(&mut self) -> Result<u32> {
        // LEB128
        let mut val = 0;
        for i in 0..5 {
            let byte = self.get_u8_le()?;
            val |= (u32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(val);
            }
        }
        Err(Error::VarIntTooLarge)
    }

    pub fn get_var_i32(&mut self) -> Result<i32> {
        let val = self.get_var_u32()?;
        // ZigZag
        Ok(((val >> 1) as i32) ^ -((val as i32) & 1))
    }

    pub fn get_var_u64(&mut self) -> Result<u64> {
        // LEB128
        let mut val = 0;
        for i in 0..10 {
            let byte = self.get_u8_le()?;
            val |= (u64::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(val);
            }
        }
        Err(Error::VarLongTooLarge)
    }

    pub fn get_var_i64(&mut self) -> Result<i64> {
        let val = self.get_var_u64()?;
        // ZigZag
        Ok(((val >> 1) as i64) ^ -((val as i64) & 1))
    }

    pub fn read_boxed_slice(&mut self, count: usize) -> Result<Box<[u8]>> {
        let mut buf = vec![0u8; count];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(buf.into())
    }

    pub fn read_vec(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; count];
        self.reader
            .read_exact(&mut buf)
            .map_err(Error::Incomplete)?;

        Ok(buf)
    }
}

pub struct Deserializer<R: Read + Seek> {
    input: NbtReadHelper<R>,
    tag_to_deserialize_stack: Option<u8>,
    // Yes, this breaks with recursion. Just an attempt at a sanity check
    in_list: bool,
    is_named: bool,
    bedrock: bool,
}

impl<R: Read + Seek> Deserializer<R> {
    pub const fn new(input: R, is_named: bool, bedrock: bool) -> Self {
        Self {
            input: NbtReadHelper { reader: input },
            tag_to_deserialize_stack: None,
            in_list: false,
            is_named,
            bedrock,
        }
    }
}

/// Deserializes struct using Serde Deserializer from normal NBT
pub fn from_bytes<'a, T: Deserialize<'a>>(r: impl Read + Seek) -> Result<T> {
    let mut deserializer = Deserializer::new(r, true, false);
    T::deserialize(&mut deserializer)
}

/// Deserializes struct using Serde Deserializer from network NBT
pub fn from_bytes_unnamed<'a, T: Deserialize<'a>>(r: impl Read + Seek) -> Result<T> {
    let mut deserializer = Deserializer::new(r, false, false);
    T::deserialize(&mut deserializer)
}

/// Deserializes struct using Serde Deserializer from Bedrock network NBT
pub fn from_bytes_bedrock<'a, T: Deserialize<'a>>(r: impl Read + Seek) -> Result<T> {
    let mut deserializer = Deserializer::new(r, true, true);
    T::deserialize(&mut deserializer)
}

impl<'de, R: Read + Seek> de::Deserializer<'de> for &mut Deserializer<R> {
    type Error = Error;

    forward_to_deserialize_any! {
        i8 i16 i32 i64 f32 f64 char str string unit unit_struct seq tuple tuple_struct
        bytes newtype_struct byte_buf
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let Some(tag) = self.tag_to_deserialize_stack else {
            return Err(Error::SerdeError("Ignoring nothing!".to_string()));
        };

        if self.bedrock {
            NbtTag::skip_data_bedrock(&mut self.input, tag)?;
        } else {
            NbtTag::skip_data(&mut self.input, tag)?;
        }

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
                    LIST_ID => {
                        if self.bedrock {
                            self.input.get_u8_le()?
                        } else {
                            self.input.get_u8_be()?
                        }
                    }
                    INT_ARRAY_ID => INT_ID,
                    LONG_ARRAY_ID => LONG_ID,
                    BYTE_ARRAY_ID => BYTE_ID,
                    _ => unreachable!(),
                };

                let remaining_values = if self.bedrock {
                    self.input.get_var_i32()?
                } else {
                    self.input.get_i32_be()?
                };
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
            let value = if self.bedrock {
                self.input.get_u8_le()?
            } else {
                self.input.get_u8_be()?
            };
            visitor.visit_u8::<Error>(value)
        } else {
            Err(Error::UnsupportedType(
                "u8; NBT only supports signed values".to_string(),
            ))
        }
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = if self.bedrock {
            self.input.get_i16_le()?
        } else {
            self.input.get_i16_be()?
        };
        visitor.visit_i16::<Error>(value)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = if self.bedrock {
            // TODO: figure out of this makes sense or if it should be an error
            self.input.get_var_u32()? as i32
        } else {
            self.input.get_i32_be()?
        };
        visitor.visit_i32::<Error>(value)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = if self.bedrock {
            // TODO: figure out of this makes sense or if it should be an error
            self.input.get_var_u64()? as i64
        } else {
            self.input.get_i64_be()?
        };
        visitor.visit_i64::<Error>(value)
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.tag_to_deserialize_stack.unwrap() == BYTE_ID {
            let value = if self.bedrock {
                self.input.get_u8_le()?
            } else {
                self.input.get_u8_be()?
            };
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
        let variant = if self.bedrock {
            get_nbt_string_bedrock(&mut self.input)?
        } else {
            get_nbt_string(&mut self.input)?
        };
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
            let next_byte = if self.bedrock {
                self.input.get_u8_le()?
            } else {
                self.input.get_u8_be()?
            };
            if next_byte != COMPOUND_ID {
                return Err(Error::NoRootCompound(next_byte));
            }

            if self.is_named {
                if self.bedrock {
                    // Consume struct name, similar to get_nbt_string_bedrock but without String::from_utf8
                    let length = self.input.get_var_u32()? as usize;
                    let _ = self.input.read_vec(length)?;
                } else {
                    // Consume struct name, similar to get_nbt_string but without cesu8::from_java_cesu8
                    let length = self.input.get_u16_be()? as usize;
                    let _ = self.input.read_boxed_slice(length)?;
                }
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
        let str = if self.bedrock {
            get_nbt_string_bedrock(&mut self.input)?
        } else {
            get_nbt_string(&mut self.input)?
        };
        visitor.visit_string(str)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

struct CompoundAccess<'a, R: Read + Seek> {
    de: &'a mut Deserializer<R>,
}

impl<'de, R: Read + Seek> MapAccess<'de> for CompoundAccess<'_, R> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        let tag = if self.de.bedrock {
            self.de.input.get_u8_le()?
        } else {
            self.de.input.get_u8_be()?
        };
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

struct MapKey<'a, R: Read + Seek> {
    de: &'a mut Deserializer<R>,
}

impl<'de, R: Read + Seek> de::Deserializer<'de> for MapKey<'_, R> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let key = if self.de.bedrock {
            get_nbt_string_bedrock(&mut self.de.input)?
        } else {
            get_nbt_string(&mut self.de.input)?
        };
        visitor.visit_string(key)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit unit_struct seq tuple tuple_struct map
        struct identifier ignored_any bytes enum newtype_struct byte_buf option
    }
}

struct ListAccess<'a, R: Read + Seek> {
    de: &'a mut Deserializer<R>,
    remaining_values: usize,
    list_type: u8,
}

impl<'de, R: Read + Seek> SeqAccess<'de> for ListAccess<'_, R> {
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

use std::borrow::Cow;
use std::cell::RefCell;
use std::io::{Cursor, Seek, SeekFrom};

use crate::{
    BYTE_ARRAY_ID, BYTE_ID, COMPOUND_ID, END_ID, Error, INT_ARRAY_ID, INT_ID, LIST_ID,
    LONG_ARRAY_ID, LONG_ID, MAX_ARRAY_LENGTH, NbtTag, io,
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

pub trait NbtDataSource<'a> {
    fn read_u8(&mut self) -> Result<u8>;
    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()>;
    fn seek_relative(&mut self, offset: i64) -> Result<()>;
    fn read_string(&mut self, len: usize) -> Result<Cow<'a, str>>;
    fn read_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>>;
}

pub struct NbtStreamReader<R>(pub R);

impl<'a, R: Read + Seek> NbtDataSource<'a> for NbtStreamReader<R> {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.0.read_exact(&mut buf).map_err(Error::Incomplete)?;
        Ok(buf[0])
    }

    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        self.0.read_exact(buf).map_err(Error::Incomplete)
    }

    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        self.0
            .seek(SeekFrom::Current(offset))
            .map_err(Error::Incomplete)?;
        Ok(())
    }

    fn read_string(&mut self, len: usize) -> Result<Cow<'a, str>> {
        let mut buf = vec![0u8; len];
        self.0.read_exact(&mut buf).map_err(Error::Incomplete)?;
        let string = cesu8::from_java_cesu8(&buf).map_err(|_| Error::Cesu8DecodingError)?;
        Ok(Cow::Owned(string.into_owned()))
    }

    fn read_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        let mut buf = vec![0u8; len];
        self.0.read_exact(&mut buf).map_err(Error::Incomplete)?;
        let i8_buf: Vec<i8> = buf.into_iter().map(|b| b as i8).collect();
        Ok(Cow::Owned(i8_buf))
    }
}

impl<'a> NbtDataSource<'a> for Cursor<&'a [u8]> {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).map_err(Error::Incomplete)?;
        Ok(buf[0])
    }

    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        self.read_exact(buf).map_err(Error::Incomplete)
    }

    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        self.seek(SeekFrom::Current(offset))
            .map_err(Error::Incomplete)?;
        Ok(())
    }

    fn read_string(&mut self, len: usize) -> Result<Cow<'a, str>> {
        let pos = self.position() as usize;
        let data_len = self.get_ref().len();
        if pos + len > data_len {
            return Err(Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF",
            )));
        }
        self.set_position((pos + len) as u64);
        let data = self.get_ref();
        let slice = &data[pos..pos + len];
        if let Ok(s) = std::str::from_utf8(slice) {
            Ok(Cow::Borrowed(s))
        } else {
            let string = cesu8::from_java_cesu8(slice).map_err(|_| Error::Cesu8DecodingError)?;
            Ok(string)
        }
    }

    fn read_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        let pos = self.position() as usize;
        let data_len = self.get_ref().len();
        if pos + len > data_len {
            return Err(Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF",
            )));
        }
        self.set_position((pos + len) as u64);
        let data = self.get_ref();
        let slice = &data[pos..pos + len];
        let i8_slice = unsafe { std::slice::from_raw_parts(slice.as_ptr().cast::<i8>(), len) };
        Ok(Cow::Borrowed(i8_slice))
    }
}

impl<'a, S: NbtDataSource<'a> + ?Sized> NbtDataSource<'a> for &mut S {
    fn read_u8(&mut self) -> Result<u8> {
        (**self).read_u8()
    }
    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        (**self).read_bytes(buf)
    }
    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        (**self).seek_relative(offset)
    }
    fn read_string(&mut self, len: usize) -> Result<Cow<'a, str>> {
        (**self).read_string(len)
    }
    fn read_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        (**self).read_byte_array(len)
    }
}

impl<'a> NbtDataSource<'a> for Cursor<Vec<u8>> {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).map_err(Error::Incomplete)?;
        Ok(buf[0])
    }

    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        self.read_exact(buf).map_err(Error::Incomplete)
    }

    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        self.seek(SeekFrom::Current(offset))
            .map_err(Error::Incomplete)?;
        Ok(())
    }

    fn read_string(&mut self, len: usize) -> Result<Cow<'a, str>> {
        let pos = self.position() as usize;
        let data_len = self.get_ref().len();
        if pos + len > data_len {
            return Err(Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF",
            )));
        }
        self.set_position((pos + len) as u64);
        let data = self.get_ref();
        let slice = &data[pos..pos + len];
        let string = cesu8::from_java_cesu8(slice).map_err(|_| Error::Cesu8DecodingError)?;
        Ok(Cow::Owned(string.into_owned()))
    }

    fn read_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        let pos = self.position() as usize;
        let data_len = self.get_ref().len();
        if pos + len > data_len {
            return Err(Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF",
            )));
        }
        self.set_position((pos + len) as u64);
        let data = self.get_ref();
        let slice = &data[pos..pos + len];
        let i8_slice = unsafe { std::slice::from_raw_parts(slice.as_ptr().cast::<i8>(), len) };
        Ok(Cow::Owned(i8_slice.to_vec()))
    }
}

pub trait NbtReadHelper<'a> {
    type Reader: NbtDataSource<'a>;

    fn reader(&mut self) -> &mut Self::Reader;

    fn skip_bytes(&mut self, count: i64) -> Result<()> {
        self.reader().seek_relative(count)
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
    fn get_string(&mut self) -> Result<Cow<'a, str>>;
    fn get_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>>;
}

pub struct NbtReadHelperJava<D> {
    reader: D,
}

impl<D> NbtReadHelperJava<D> {
    pub const fn new(r: D) -> Self {
        Self { reader: r }
    }
}

pub struct NbtReadHelperBedrock<D> {
    reader: D,
}

impl<D> NbtReadHelperBedrock<D> {
    pub const fn new(r: D) -> Self {
        Self { reader: r }
    }
}

impl<'a, D: NbtDataSource<'a>> NbtReadHelperJava<D> {
    fn get_string_len(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.reader.read_bytes(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }
}

impl<'a, D: NbtDataSource<'a>> NbtReadHelper<'a> for NbtReadHelperJava<D> {
    type Reader = D;

    fn reader(&mut self) -> &mut D {
        &mut self.reader
    }

    fn skip_string(&mut self) -> Result<()> {
        let len = self.get_string_len()? as i64;
        self.skip_bytes(len)
    }

    fn get_u8(&mut self) -> Result<u8> {
        self.reader.read_u8()
    }
    fn get_i8(&mut self) -> Result<i8> {
        Ok(self.reader.read_u8()? as i8)
    }
    fn get_i16(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.reader.read_bytes(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }
    fn get_i32(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.reader.read_bytes(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }
    fn get_i64(&mut self) -> Result<i64> {
        let mut buf = [0u8; 8];
        self.reader.read_bytes(&mut buf)?;
        Ok(i64::from_be_bytes(buf))
    }
    fn get_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.reader.read_bytes(&mut buf)?;
        Ok(f32::from_be_bytes(buf))
    }
    fn get_f64(&mut self) -> Result<f64> {
        let mut buf = [0u8; 8];
        self.reader.read_bytes(&mut buf)?;
        Ok(f64::from_be_bytes(buf))
    }

    fn get_string(&mut self) -> Result<Cow<'a, str>> {
        let len = self.get_string_len()? as usize;
        self.reader.read_string(len)
    }

    fn get_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        self.reader.read_byte_array(len)
    }
}

impl<'a, D: NbtDataSource<'a>> NbtReadHelperBedrock<D> {
    fn get_u8(&mut self) -> Result<u8> {
        self.reader.read_u8()
    }

    fn get_var_u32(&mut self) -> Result<u32> {
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
        Ok(((val >> 1) as i32) ^ -((val as i32) & 1))
    }

    fn get_var_u64(&mut self) -> Result<u64> {
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
        Ok(((val >> 1) as i64) ^ -((val as i64) & 1))
    }

    fn get_string_len(&mut self) -> Result<u32> {
        self.get_var_u32()
    }
}

impl<'a, D: NbtDataSource<'a>> NbtReadHelper<'a> for NbtReadHelperBedrock<D> {
    type Reader = D;

    fn reader(&mut self) -> &mut D {
        &mut self.reader
    }

    fn skip_string(&mut self) -> Result<()> {
        let len = self.get_string_len()? as i64;
        self.skip_bytes(len)
    }

    fn get_u8(&mut self) -> Result<u8> {
        self.reader.read_u8()
    }
    fn get_i8(&mut self) -> Result<i8> {
        Ok(self.reader.read_u8()? as i8)
    }
    fn get_i16(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.reader.read_bytes(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }
    fn get_i32(&mut self) -> Result<i32> {
        self.get_var_i32()
    }
    fn get_i64(&mut self) -> Result<i64> {
        self.get_var_i64()
    }
    fn get_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.reader.read_bytes(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }
    fn get_f64(&mut self) -> Result<f64> {
        let mut buf = [0u8; 8];
        self.reader.read_bytes(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    fn get_string(&mut self) -> Result<Cow<'a, str>> {
        let len = self.get_string_len()? as usize;
        self.reader.read_string(len)
    }

    fn get_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        self.reader.read_byte_array(len)
    }
}

pub struct Deserializer<R> {
    input: R,
    tag_to_deserialize_stack: Option<u8>,
    in_list: bool,
    is_named: bool,
}

impl<R> Deserializer<R> {
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
    let mut deserializer = Deserializer::new(NbtReadHelperJava::new(NbtStreamReader(r)), true);
    T::deserialize(&mut deserializer)
}

/// Deserializes struct using Serde Deserializer from network NBT
pub fn from_bytes_unnamed<'a, T: Deserialize<'a>>(r: impl Read + Seek) -> Result<T> {
    let mut deserializer = Deserializer::new(NbtReadHelperJava::new(NbtStreamReader(r)), false);
    T::deserialize(&mut deserializer)
}

/// Deserializes struct using Serde Deserializer from Bedrock network NBT
pub fn from_bytes_bedrock<'a, T: Deserialize<'a>>(r: impl Read + Seek) -> Result<T> {
    let mut deserializer = Deserializer::new(NbtReadHelperBedrock::new(NbtStreamReader(r)), true);
    T::deserialize(&mut deserializer)
}

/// Deserializes struct using Serde Deserializer from a normal NBT slice (zero-allocation)
pub fn from_slice<'a, T: Deserialize<'a>>(slice: &'a [u8]) -> Result<T> {
    let mut deserializer = Deserializer::new(NbtReadHelperJava::new(Cursor::new(slice)), true);
    T::deserialize(&mut deserializer)
}

/// Deserializes struct using Serde Deserializer from a network NBT slice (zero-allocation)
pub fn from_slice_unnamed<'a, T: Deserialize<'a>>(slice: &'a [u8]) -> Result<T> {
    let mut deserializer = Deserializer::new(NbtReadHelperJava::new(Cursor::new(slice)), false);
    T::deserialize(&mut deserializer)
}

/// Deserializes struct using Serde Deserializer from a Bedrock network NBT slice (zero-allocation)
pub fn from_slice_bedrock<'a, T: Deserialize<'a>>(slice: &'a [u8]) -> Result<T> {
    let mut deserializer = Deserializer::new(NbtReadHelperBedrock::new(Cursor::new(slice)), true);
    T::deserialize(&mut deserializer)
}

impl<'de, R: NbtReadHelper<'de>> de::Deserializer<'de> for &mut Deserializer<R> {
    type Error = Error;

    forward_to_deserialize_any! {
        i8 i16 i32 i64 f32 f64 char unit unit_struct seq tuple tuple_struct
        newtype_struct byte_buf
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
                    _ => return Err(Error::SerdeError("Unreachable state reached".to_string())),
                };

                let remaining_values = self.input.get_i32()?;
                if remaining_values < 0 {
                    return Err(Error::NegativeLength(remaining_values));
                }

                let remaining_values = remaining_values as usize;
                if remaining_values > MAX_ARRAY_LENGTH {
                    return Err(Error::LargeLength(remaining_values));
                }

                set_curr_visitor_seq_list_id(Some(list_type));
                let result = visitor.visit_seq(ListAccess {
                    de: self,
                    list_type,
                    remaining_values,
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
                    NbtTag::String(value) => visitor.visit_string::<Error>(value.into_string())?,
                    _ => return Err(Error::SerdeError("Unreachable state reached".to_string())),
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
        let value = self.input.get_i32()?;
        visitor.visit_i32::<Error>(value)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
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

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.input.get_string()? {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            Cow::Owned(s) => visitor.visit_string(s),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let tag_id = self.tag_to_deserialize_stack.unwrap_or(BYTE_ARRAY_ID);
        if tag_id == BYTE_ARRAY_ID {
            let len = self.input.get_i32()?;
            if len < 0 {
                return Err(Error::NegativeLength(len));
            }
            let slice = self.input.get_byte_array(len as usize)?;
            let u8_slice: &'de [u8] =
                unsafe { std::slice::from_raw_parts(slice.as_ptr().cast::<u8>(), slice.len()) };
            match slice {
                Cow::Borrowed(_) => visitor.visit_borrowed_bytes(u8_slice),
                Cow::Owned(_) => visitor.visit_bytes(u8_slice),
            }
        } else {
            self.deserialize_any(visitor)
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        match self.input.get_string()? {
            Cow::Borrowed(s) => visitor.visit_enum(s.into_deserializer()),
            Cow::Owned(s) => visitor.visit_enum(s.into_deserializer()),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
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
        self.deserialize_str(visitor)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

struct CompoundAccess<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'de, R: NbtReadHelper<'de>> MapAccess<'de> for CompoundAccess<'_, R> {
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

struct MapKey<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'de, R: NbtReadHelper<'de>> de::Deserializer<'de> for MapKey<'_, R> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.de.input.get_string()? {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            Cow::Owned(s) => visitor.visit_string(s),
        }
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit unit_struct seq tuple tuple_struct map
        struct identifier ignored_any bytes enum newtype_struct byte_buf option
    }
}

struct ListAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    remaining_values: usize,
    list_type: u8,
}

impl<'de, R: NbtReadHelper<'de>> SeqAccess<'de> for ListAccess<'_, R> {
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

use core::str;
use std::borrow::Cow;
use std::io::{Read, Write};

use crate::{
    FixedBitSet,
    codec::{
        bit_set::BitSet, var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong,
    },
};

use pumpkin_nbt::{serializer::NbtWriteHelperJava, tag::NbtTag};
use pumpkin_util::math::position::BlockPos;
use thiserror::Error;

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

impl serde::de::Error for ReadingError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

#[derive(Debug, Error)]
pub enum WritingError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serde failure: {0}")]
    Serde(String),
    #[error("Failed to serialize packet: {0}")]
    Message(String),
}

impl serde::ser::Error for WritingError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Serde(msg.to_string())
    }
}

pub trait NetworkReadExt {
    fn get_i8(&mut self) -> Result<i8, ReadingError>;
    fn get_u8(&mut self) -> Result<u8, ReadingError>;

    fn get_i16_be(&mut self) -> Result<i16, ReadingError>;
    fn get_u16_be(&mut self) -> Result<u16, ReadingError>;
    fn get_i32_be(&mut self) -> Result<i32, ReadingError>;
    fn get_u32_be(&mut self) -> Result<u32, ReadingError>;
    fn get_i64_be(&mut self) -> Result<i64, ReadingError>;
    fn get_u64_be(&mut self) -> Result<u64, ReadingError>;
    fn get_f32_be(&mut self) -> Result<f32, ReadingError>;
    fn get_f64_be(&mut self) -> Result<f64, ReadingError>;
    fn get_i128_be(&mut self) -> Result<i128, ReadingError>;
    fn get_u128_be(&mut self) -> Result<u128, ReadingError>;

    #[inline]
    fn get_i16(&mut self) -> Result<i16, ReadingError> {
        self.get_i16_be()
    }
    #[inline]
    fn get_u16(&mut self) -> Result<u16, ReadingError> {
        self.get_u16_be()
    }
    #[inline]
    fn get_i32(&mut self) -> Result<i32, ReadingError> {
        self.get_i32_be()
    }
    #[inline]
    fn get_u32(&mut self) -> Result<u32, ReadingError> {
        self.get_u32_be()
    }
    #[inline]
    fn get_i64(&mut self) -> Result<i64, ReadingError> {
        self.get_i64_be()
    }
    #[inline]
    fn get_u64(&mut self) -> Result<u64, ReadingError> {
        self.get_u64_be()
    }
    #[inline]
    fn get_f32(&mut self) -> Result<f32, ReadingError> {
        self.get_f32_be()
    }
    #[inline]
    fn get_f64(&mut self) -> Result<f64, ReadingError> {
        self.get_f64_be()
    }

    fn read_bytes_to_buf(&mut self, buf: &mut [u8]) -> Result<(), ReadingError>;

    fn get_bool(&mut self) -> Result<bool, ReadingError>;
    fn get_var_int(&mut self) -> Result<VarInt, ReadingError>;
    fn get_var_uint(&mut self) -> Result<VarUInt, ReadingError>;
    fn get_var_long(&mut self) -> Result<VarLong, ReadingError>;
    fn get_var_ulong(&mut self) -> Result<VarULong, ReadingError>;
    fn get_str_bounded(&mut self, bound: usize) -> Result<Box<str>, ReadingError>;
    #[inline]
    fn get_str(&mut self) -> Result<Box<str>, ReadingError> {
        self.get_str_bounded(i32::MAX as usize)
    }
    fn get_uuid(&mut self) -> Result<uuid::Uuid, ReadingError>;
    fn get_fixed_bitset(&mut self, bits: usize) -> Result<FixedBitSet, ReadingError>;

    #[inline]
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

    #[inline]
    fn get_list<G>(
        &mut self,
        parse: impl Fn(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Vec<G>, ReadingError> {
        const MAX_LIST_SIZE: usize = 65536;

        let len = self.get_var_int()?.0 as usize;
        if len > MAX_LIST_SIZE {
            return Err(ReadingError::TooLarge(format!(
                "List length {len} exceeds limit"
            )));
        }
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(parse(self)?);
        }
        Ok(list)
    }
}

pub trait NetworkReadSliceExt<'a> {
    fn get_str_borrowed(&mut self) -> Result<&'a str, ReadingError>;
    fn get_str_bounded_borrowed(&mut self, bound: usize) -> Result<&'a str, ReadingError>;
    fn read_slice_borrowed(&mut self, count: usize) -> Result<&'a [u8], ReadingError>;
    fn read_remaining_slice_borrowed(&mut self, bound: usize) -> Result<&'a [u8], ReadingError>;

    #[inline]
    fn read_cow_slice_borrowed(&mut self, count: usize) -> Result<Cow<'a, [u8]>, ReadingError> {
        Ok(Cow::Borrowed(self.read_slice_borrowed(count)?))
    }

    #[inline]
    fn read_remaining_to_cow_slice_borrowed(
        &mut self,
        bound: usize,
    ) -> Result<Cow<'a, [u8]>, ReadingError> {
        Ok(Cow::Borrowed(self.read_remaining_slice_borrowed(bound)?))
    }

    #[inline]
    fn get_cow_str_borrowed(&mut self) -> Result<Cow<'a, str>, ReadingError> {
        self.get_cow_str_bounded_borrowed(i32::MAX as usize)
    }

    #[inline]
    fn get_cow_str_bounded_borrowed(&mut self, bound: usize) -> Result<Cow<'a, str>, ReadingError> {
        Ok(Cow::Borrowed(self.get_str_bounded_borrowed(bound)?))
    }
}

impl<'a> NetworkReadSliceExt<'a> for &'a [u8] {
    #[inline]
    fn read_slice_borrowed(&mut self, count: usize) -> Result<&'a [u8], ReadingError> {
        if self.len() < count {
            return Err(ReadingError::Incomplete(format!(
                "EOF, Tried to read {count} bytes but only {} bytes left",
                self.len()
            )));
        }
        let (head, tail) = self.split_at(count);
        *self = tail;
        Ok(head)
    }

    #[inline]
    fn read_remaining_slice_borrowed(&mut self, bound: usize) -> Result<&'a [u8], ReadingError> {
        if self.len() > bound {
            return Err(ReadingError::TooLarge(
                "Read remaining too long".to_string(),
            ));
        }
        let slice = *self;
        *self = &[];
        Ok(slice)
    }

    #[inline]
    fn get_str_bounded_borrowed(&mut self, bound: usize) -> Result<&'a str, ReadingError> {
        let bytes_len = self.get_var_uint()?.0 as usize;

        let maximum_utf8_bytes = bound.saturating_mul(3);
        if bytes_len > maximum_utf8_bytes {
            return Err(ReadingError::TooLarge(format!(
                "string has too many bytes ({bytes_len} > {maximum_utf8_bytes})"
            )));
        }

        let bytes = self.read_slice_borrowed(bytes_len)?;
        let string =
            std::str::from_utf8(bytes).map_err(|e| ReadingError::Message(e.to_string()))?;

        if string.encode_utf16().nth(bound).is_some() {
            return Err(ReadingError::TooLarge(format!(
                "string has too many UTF-16 characters (more than the maximum limit {bound})"
            )));
        }

        Ok(string)
    }

    #[inline]
    fn get_str_borrowed(&mut self) -> Result<&'a str, ReadingError> {
        self.get_str_bounded_borrowed(i32::MAX as usize)
    }
}

impl<'a, R: NetworkReadSliceExt<'a> + ?Sized> NetworkReadSliceExt<'a> for &mut R {
    #[inline]
    fn get_str_borrowed(&mut self) -> Result<&'a str, ReadingError> {
        (**self).get_str_borrowed()
    }
    #[inline]
    fn get_str_bounded_borrowed(&mut self, bound: usize) -> Result<&'a str, ReadingError> {
        (**self).get_str_bounded_borrowed(bound)
    }
    #[inline]
    fn read_slice_borrowed(&mut self, count: usize) -> Result<&'a [u8], ReadingError> {
        (**self).read_slice_borrowed(count)
    }
    #[inline]
    fn read_remaining_slice_borrowed(&mut self, bound: usize) -> Result<&'a [u8], ReadingError> {
        (**self).read_remaining_slice_borrowed(bound)
    }
}

macro_rules! get_number_be {
    ($name:ident, $type:ty) => {
        #[inline]
        fn $name(&mut self) -> Result<$type, ReadingError> {
            let mut buf = [0u8; std::mem::size_of::<$type>()];
            self.read_exact(&mut buf)
                .map_err(|err| ReadingError::Incomplete(err.to_string()))?;
            Ok(<$type>::from_be_bytes(buf))
        }
    };
}

impl<R: Read> NetworkReadExt for R {
    get_number_be!(get_u8, u8);
    get_number_be!(get_i8, i8);

    get_number_be!(get_i16_be, i16);
    get_number_be!(get_u16_be, u16);
    get_number_be!(get_i32_be, i32);
    get_number_be!(get_u32_be, u32);
    get_number_be!(get_i64_be, i64);
    get_number_be!(get_u64_be, u64);
    get_number_be!(get_i128_be, i128);
    get_number_be!(get_u128_be, u128);
    get_number_be!(get_f32_be, f32);
    get_number_be!(get_f64_be, f64);

    #[inline]
    fn read_bytes_to_buf(&mut self, buf: &mut [u8]) -> Result<(), ReadingError> {
        self.read_exact(buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))
    }

    #[inline]
    fn get_bool(&mut self) -> Result<bool, ReadingError> {
        let byte = self.get_u8()?;
        Ok(byte != 0)
    }

    #[inline]
    fn get_var_int(&mut self) -> Result<VarInt, ReadingError> {
        VarInt::decode(self)
    }
    #[inline]
    fn get_var_uint(&mut self) -> Result<VarUInt, ReadingError> {
        VarUInt::decode(self)
    }

    #[inline]
    fn get_var_long(&mut self) -> Result<VarLong, ReadingError> {
        VarLong::decode(self)
    }

    #[inline]
    fn get_var_ulong(&mut self) -> Result<VarULong, ReadingError> {
        VarULong::decode(self)
    }

    #[inline]
    fn get_str_bounded(&mut self, bound: usize) -> Result<Box<str>, ReadingError> {
        let bytes_len = self.get_var_uint()?.0 as usize;

        // We treat `bound` as the maximum number of Java `char`s allowed.

        // First, check if there are too many bytes to even fit in the UTF-16 bound.
        // 1 Java `char` takes a maximum of 3 bytes in UTF-8:
        let maximum_utf8_bytes = bound.saturating_mul(3);
        if bytes_len > maximum_utf8_bytes {
            return Err(ReadingError::TooLarge(format!(
                "string has too many bytes ({bytes_len} > {maximum_utf8_bytes})"
            )));
        }

        if bytes_len <= 128 {
            let mut stack_buf = [0u8; 128];
            let slice = &mut stack_buf[..bytes_len];
            self.read_exact(slice)
                .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

            let string =
                std::str::from_utf8(slice).map_err(|e| ReadingError::Message(e.to_string()))?;

            if string.encode_utf16().nth(bound).is_some() {
                return Err(ReadingError::TooLarge(format!(
                    "string has too many UTF-16 characters (more than the maximum limit {bound})"
                )));
            }

            Ok(string.into())
        } else {
            let mut data = vec![0u8; bytes_len];
            self.read_bytes_to_buf(&mut data)?;
            let string =
                std::str::from_utf8(&data).map_err(|e| ReadingError::Message(e.to_string()))?;

            if string.encode_utf16().nth(bound).is_some() {
                return Err(ReadingError::TooLarge(format!(
                    "string has too many UTF-16 characters (more than the maximum limit {bound})"
                )));
            }

            Ok(string.into())
        }
    }

    #[inline]
    fn get_uuid(&mut self) -> Result<uuid::Uuid, ReadingError> {
        let mut bytes = [0u8; 16];
        self.read_exact(&mut bytes)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;
        Ok(uuid::Uuid::from_bytes(bytes))
    }

    #[inline]
    fn get_fixed_bitset(&mut self, bits: usize) -> Result<FixedBitSet, ReadingError> {
        let byte_count = bits.div_ceil(8);
        let mut bytes = vec![0u8; byte_count];
        self.read_bytes_to_buf(&mut bytes)?;
        Ok(bytes.into_boxed_slice())
    }
}

#[inline]
pub fn read_remaining_bytes(read: &mut impl Read, bound: usize) -> Result<Box<[u8]>, ReadingError> {
    let mut return_buf = Vec::with_capacity(bound.min(1024));

    // Take one extra byte to check for exceeding bound
    read.take(bound as u64 + 1)
        .read_to_end(&mut return_buf)
        .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

    if return_buf.len() > bound {
        return Err(ReadingError::TooLarge(
            "Read remaining too long".to_string(),
        ));
    }

    Ok(return_buf.into_boxed_slice())
}

pub trait NetworkWriteExt {
    fn write_i8(&mut self, data: i8) -> Result<(), WritingError>;
    fn write_u8(&mut self, data: u8) -> Result<(), WritingError>;
    fn write_i16_be(&mut self, data: i16) -> Result<(), WritingError>;
    fn write_u16_be(&mut self, data: u16) -> Result<(), WritingError>;
    fn write_i32_be(&mut self, data: i32) -> Result<(), WritingError>;
    fn write_u32_be(&mut self, data: u32) -> Result<(), WritingError>;
    fn write_i64_be(&mut self, data: i64) -> Result<(), WritingError>;
    fn write_u64_be(&mut self, data: u64) -> Result<(), WritingError>;
    fn write_f32_be(&mut self, data: f32) -> Result<(), WritingError>;
    fn write_f64_be(&mut self, data: f64) -> Result<(), WritingError>;
    fn write_slice(&mut self, data: &[u8]) -> Result<(), WritingError>;

    fn write_i16(&mut self, data: i16) -> Result<(), WritingError> {
        self.write_i16_be(data)
    }
    fn write_u16(&mut self, data: u16) -> Result<(), WritingError> {
        self.write_u16_be(data)
    }
    fn write_i32(&mut self, data: i32) -> Result<(), WritingError> {
        self.write_i32_be(data)
    }
    fn write_u32(&mut self, data: u32) -> Result<(), WritingError> {
        self.write_u32_be(data)
    }
    fn write_i64(&mut self, data: i64) -> Result<(), WritingError> {
        self.write_i64_be(data)
    }
    fn write_u64(&mut self, data: u64) -> Result<(), WritingError> {
        self.write_u64_be(data)
    }
    fn write_f32(&mut self, data: f32) -> Result<(), WritingError> {
        self.write_f32_be(data)
    }
    fn write_f64(&mut self, data: f64) -> Result<(), WritingError> {
        self.write_f64_be(data)
    }

    fn put_var_int(&mut self, data: &VarInt) -> Result<(), WritingError> {
        self.write_var_int(data)
    }
    fn put_i32(&mut self, data: i32) -> Result<(), WritingError> {
        self.write_i32(data)
    }
    fn put_bool(&mut self, data: bool) -> Result<(), WritingError> {
        self.write_bool(data)
    }

    fn write_bool(&mut self, data: bool) -> Result<(), WritingError> {
        if data {
            self.write_u8(1)
        } else {
            self.write_u8(0)
        }
    }
    fn write_fixed_bitset(&mut self, bits: usize, bit_set: FixedBitSet)
    -> Result<(), WritingError>;
    fn write_var_int(&mut self, data: &VarInt) -> Result<(), WritingError>;
    fn write_var_uint(&mut self, data: &VarUInt) -> Result<(), WritingError>;
    fn write_var_long(&mut self, data: &VarLong) -> Result<(), WritingError>;
    fn write_string_bounded(&mut self, data: &str, bound: usize) -> Result<(), WritingError>;
    fn write_string(&mut self, data: &str) -> Result<(), WritingError>;
    fn write_block_pos(&mut self, pos: &BlockPos) -> Result<(), WritingError>;

    fn write_uuid(&mut self, data: &uuid::Uuid) -> Result<(), WritingError> {
        let (first, second) = data.as_u64_pair();
        self.write_u64_be(first)?;
        self.write_u64_be(second)
    }

    fn write_bitset(&mut self, bitset: &BitSet) -> Result<(), WritingError>;

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
        self.write_var_int(&(list.len() as i32).into())?;

        for data in list {
            writer(self, data)?;
        }

        Ok(())
    }

    fn write_nbt(&mut self, data: NbtTag) -> Result<(), WritingError>;
}

macro_rules! write_number_be {
    ($name:ident, $type:ty) => {
        fn $name(&mut self, data: $type) -> Result<(), WritingError> {
            self.write_all(&data.to_be_bytes())
                .map_err(WritingError::IoError)
        }
    };
}

impl<W: Write> NetworkWriteExt for W {
    fn write_i8(&mut self, data: i8) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_u8(&mut self, data: u8) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    write_number_be!(write_i16_be, i16);
    write_number_be!(write_u16_be, u16);
    write_number_be!(write_i32_be, i32);
    write_number_be!(write_u32_be, u32);
    write_number_be!(write_i64_be, i64);
    write_number_be!(write_u64_be, u64);
    write_number_be!(write_f32_be, f32);
    write_number_be!(write_f64_be, f64);

    fn write_slice(&mut self, data: &[u8]) -> Result<(), WritingError> {
        self.write_all(data).map_err(WritingError::IoError)
    }

    fn write_fixed_bitset(
        &mut self,
        bits: usize,
        bit_set: FixedBitSet,
    ) -> Result<(), WritingError> {
        let new_length = bits.div_ceil(8);
        let mut new_vec = vec![0u8; new_length];
        let bytes_to_copy = std::cmp::min(bit_set.len(), new_length);

        new_vec[..bytes_to_copy].copy_from_slice(&bit_set[..bytes_to_copy]);
        self.write_slice(&new_vec)?;

        Ok(())
    }

    fn write_var_int(&mut self, data: &VarInt) -> Result<(), WritingError> {
        data.encode(self)
    }

    fn write_var_uint(&mut self, data: &VarUInt) -> Result<(), WritingError> {
        data.encode(self)
    }

    fn write_var_long(&mut self, data: &VarLong) -> Result<(), WritingError> {
        data.encode(self)
    }

    fn write_string_bounded(&mut self, data: &str, bound: usize) -> Result<(), WritingError> {
        if data.len() > bound {
            return Err(WritingError::Message(format!(
                "string length {} exceeds bound {}",
                data.len(),
                bound
            )));
        }
        self.write_var_int(&data.len().try_into().map_err(|_| {
            WritingError::Message(format!("{} isn't representable as a VarInt", data.len()))
        })?)?;

        self.write_all(data.as_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_string(&mut self, data: &str) -> Result<(), WritingError> {
        self.write_string_bounded(data, i16::MAX as usize)
    }

    fn write_block_pos(&mut self, pos: &BlockPos) -> Result<(), WritingError> {
        self.write_i64_be(pos.as_long())
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
        self.write_var_int(&(list.len() as i32).into())?;

        for data in list {
            writer(self, data)?;
        }

        Ok(())
    }

    fn write_nbt(&mut self, data: NbtTag) -> Result<(), WritingError> {
        let mut write_adaptor = NbtWriteHelperJava::new(self);
        data.serialize(&mut write_adaptor)
            .map_err(|e| WritingError::Message(e.to_string()))?;

        Ok(())
    }
}

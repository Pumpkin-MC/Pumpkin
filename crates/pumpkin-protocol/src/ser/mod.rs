use core::str;
use std::borrow::Cow;
use std::io::{Read, Write};

use crate::{
    FixedBitSet,
    codec::{
        bit_set::BitSet, var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong,
    },
};

use pumpkin_nbt::{
    compound::NbtCompound, deserializer::NbtReadHelper, serializer::NbtWriteHelperJava, tag::NbtTag,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};
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
    #[error("Packet is not supported in Minecraft version {0:?}")]
    UnsupportedVersion(JavaMinecraftVersion),
    #[error("Failed to serialize packet: {0}")]
    Message(String),
}

impl serde::ser::Error for WritingError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Serde(msg.to_string())
    }
}

struct NetworkReadDataSource<'a, R: NetworkReadExt + ?Sized>(&'a mut R);

impl<'a, R: NetworkReadExt + ?Sized> pumpkin_nbt::deserializer::NbtDataSource<'a>
    for NetworkReadDataSource<'a, R>
{
    fn read_u8(&mut self) -> Result<u8, pumpkin_nbt::Error> {
        self.0.get_u8().map_err(|e| {
            pumpkin_nbt::Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                e.to_string(),
            ))
        })
    }

    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<(), pumpkin_nbt::Error> {
        self.0.read_bytes_to_buf(buf).map_err(|e| {
            pumpkin_nbt::Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                e.to_string(),
            ))
        })
    }

    fn seek_relative(&mut self, offset: i64) -> Result<(), pumpkin_nbt::Error> {
        if offset < 0 {
            return Err(pumpkin_nbt::Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "negative seek not supported",
            )));
        }
        let mut skipped = vec![0u8; offset as usize];
        self.0.read_bytes_to_buf(&mut skipped).map_err(|e| {
            pumpkin_nbt::Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                e.to_string(),
            ))
        })
    }

    fn read_string(&mut self, len: usize) -> Result<Cow<'a, str>, pumpkin_nbt::Error> {
        let mut buf = vec![0u8; len];
        self.0.read_bytes_to_buf(&mut buf).map_err(|e| {
            pumpkin_nbt::Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                e.to_string(),
            ))
        })?;
        let string =
            cesu8::from_java_cesu8(&buf).map_err(|_| pumpkin_nbt::Error::Cesu8DecodingError)?;
        Ok(Cow::Owned(string.into_owned()))
    }

    fn read_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>, pumpkin_nbt::Error> {
        let mut buf = vec![0u8; len];
        self.0.read_bytes_to_buf(&mut buf).map_err(|e| {
            pumpkin_nbt::Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                e.to_string(),
            ))
        })?;
        let i8_buf: Vec<i8> = buf.into_iter().map(|b| b as i8).collect();
        Ok(Cow::Owned(i8_buf))
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
        self.get_str_bounded(32767)
    }
    fn get_uuid(&mut self) -> Result<uuid::Uuid, ReadingError>;
    fn get_fixed_bitset(&mut self, bits: usize) -> Result<FixedBitSet, ReadingError>;

    #[inline]
    fn get_block_pos(&mut self, version: &JavaMinecraftVersion) -> Result<BlockPos, ReadingError> {
        let val = self.get_i64_be()?;
        Ok(BlockPos::from_long_for_version(val, version))
    }

    #[inline]
    fn get_container_id(&mut self, version: &JavaMinecraftVersion) -> Result<VarInt, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_21_2 {
            self.get_var_int()
        } else {
            Ok(VarInt(i32::from(self.get_u8()?)))
        }
    }

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

    fn get_nbt_with_version(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<Option<NbtTag>, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_8 {
            let tag_id = self.get_u8()?;
            if tag_id == pumpkin_nbt::END_ID {
                return Ok(None);
            }
            let mut helper =
                pumpkin_nbt::deserializer::NbtReadHelperJava::new(NetworkReadDataSource(self));
            if *version < JavaMinecraftVersion::V_1_20_2 {
                let _name = helper
                    .get_string()
                    .map_err(|e| ReadingError::Message(e.to_string()))?;
            }
            let tag = if tag_id == pumpkin_nbt::COMPOUND_ID {
                NbtTag::Compound(
                    NbtCompound::deserialize_content(&mut helper)
                        .map_err(|e| ReadingError::Message(e.to_string()))?,
                )
            } else {
                NbtTag::deserialize_data(&mut helper, tag_id)
                    .map_err(|e| ReadingError::Message(e.to_string()))?
            };
            Ok(Some(tag))
        } else {
            let length = self.get_i16_be()?;
            if length <= 0 {
                return Ok(None);
            }
            let mut compressed = vec![0u8; length as usize];
            self.read_bytes_to_buf(&mut compressed)?;
            let mut decoder = flate2::read::GzDecoder::new(&compressed[..]);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|e| ReadingError::Message(e.to_string()))?;
            let mut cursor = std::io::Cursor::new(decompressed);
            let mut helper = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
            let tag_id = helper
                .get_u8()
                .map_err(|e| ReadingError::Message(e.to_string()))?;
            if tag_id == pumpkin_nbt::END_ID {
                return Ok(None);
            }
            let _name = helper
                .get_string()
                .map_err(|e| ReadingError::Message(e.to_string()))?;
            let tag = if tag_id == pumpkin_nbt::COMPOUND_ID {
                NbtTag::Compound(
                    NbtCompound::deserialize_content(&mut helper)
                        .map_err(|e| ReadingError::Message(e.to_string()))?,
                )
            } else {
                NbtTag::deserialize_data(&mut helper, tag_id)
                    .map_err(|e| ReadingError::Message(e.to_string()))?
            };
            Ok(Some(tag))
        }
    }

    #[inline]
    fn get_compound_nbt_with_version(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<Option<NbtCompound>, ReadingError> {
        match self.get_nbt_with_version(version)? {
            Some(NbtTag::Compound(comp)) => Ok(Some(comp)),
            Some(NbtTag::End) | None => Ok(None),
            Some(other) => Err(ReadingError::Message(format!(
                "Expected NBT compound, found tag ID {}",
                other.get_type_id()
            ))),
        }
    }
}

pub trait NetworkReadSliceExt<'a> {
    fn get_component_borrowed(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<TextComponent, ReadingError>;
    #[inline]
    fn get_component(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<TextComponent, ReadingError> {
        self.get_component_borrowed(version)
    }
    fn get_str_borrowed(&mut self) -> Result<&'a str, ReadingError>;
    fn get_str_bounded_borrowed(&mut self, bound: usize) -> Result<&'a str, ReadingError>;
    fn read_slice_borrowed(&mut self, count: usize) -> Result<&'a [u8], ReadingError>;
    fn read_remaining_slice_borrowed(&mut self, bound: usize) -> Result<&'a [u8], ReadingError>;

    fn get_nbt_borrowed(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<Option<NbtTag>, ReadingError>;

    #[inline]
    fn get_nbt(&mut self, version: &JavaMinecraftVersion) -> Result<Option<NbtTag>, ReadingError> {
        self.get_nbt_borrowed(version)
    }

    #[inline]
    fn get_compound_nbt_borrowed(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<Option<NbtCompound>, ReadingError> {
        match self.get_nbt_borrowed(version)? {
            Some(NbtTag::Compound(comp)) => Ok(Some(comp)),
            Some(NbtTag::End) | None => Ok(None),
            Some(other) => Err(ReadingError::Message(format!(
                "Expected NBT compound, found tag ID {}",
                other.get_type_id()
            ))),
        }
    }

    #[inline]
    fn get_compound_nbt(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<Option<NbtCompound>, ReadingError> {
        self.get_compound_nbt_borrowed(version)
    }

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
        self.get_cow_str_bounded_borrowed(32767)
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

        let maximum_utf8_bytes = bound.saturating_mul(3).min(crate::MAX_PACKET_DATA_SIZE);
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
        self.get_str_bounded_borrowed(32767)
    }

    #[inline]
    fn get_component_borrowed(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<TextComponent, ReadingError> {
        if *version < JavaMinecraftVersion::V_1_20_3 {
            let max_len = if *version >= JavaMinecraftVersion::V_1_13 {
                262144
            } else {
                32767
            };
            let json = self.get_str_bounded_borrowed(max_len)?;
            serde_json::from_str(json)
                .map_err(|e| ReadingError::Message(format!("Invalid component JSON: {e}")))
        } else {
            let mut cursor = std::io::Cursor::new(*self);
            let mut nbt_reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
            let nbt = NbtTag::deserialize(&mut nbt_reader)
                .map_err(|e| ReadingError::Message(format!("Invalid component NBT: {e}")))?;
            let bytes_read = cursor.position() as usize;
            *self = &self[bytes_read..];
            let json_value = nbt_tag_to_json(&nbt);
            serde_json::from_value(json_value).map_err(|e| {
                ReadingError::Message(format!("Failed to parse component from NBT: {e}"))
            })
        }
    }

    fn get_nbt_borrowed(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<Option<NbtTag>, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_8 {
            if self.is_empty() {
                return Ok(None);
            }
            if (*self)[0] == pumpkin_nbt::END_ID {
                *self = &(*self)[1..];
                return Ok(None);
            }
            let mut cursor = std::io::Cursor::new(*self);
            let mut helper = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
            let tag_id = helper
                .get_u8()
                .map_err(|e| ReadingError::Message(e.to_string()))?;
            if tag_id == pumpkin_nbt::END_ID {
                let pos = cursor.position() as usize;
                *self = &(*self)[pos..];
                return Ok(None);
            }
            if *version < JavaMinecraftVersion::V_1_20_2 {
                let _name = helper
                    .get_string()
                    .map_err(|e| ReadingError::Message(e.to_string()))?;
            }
            let tag = if tag_id == pumpkin_nbt::COMPOUND_ID {
                NbtTag::Compound(
                    NbtCompound::deserialize_content(&mut helper)
                        .map_err(|e| ReadingError::Message(e.to_string()))?,
                )
            } else {
                NbtTag::deserialize_data(&mut helper, tag_id)
                    .map_err(|e| ReadingError::Message(e.to_string()))?
            };
            let pos = cursor.position() as usize;
            *self = &(*self)[pos..];
            Ok(Some(tag))
        } else {
            let length = self.get_i16_be()?;
            if length <= 0 {
                return Ok(None);
            }
            let length = length as usize;
            if self.len() < length {
                return Err(ReadingError::Incomplete(
                    "Not enough bytes for compressed NBT".into(),
                ));
            }
            let compressed = &(*self)[..length];
            *self = &(*self)[length..];
            let mut decoder = flate2::read::GzDecoder::new(compressed);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|e| ReadingError::Message(e.to_string()))?;
            let mut cursor = std::io::Cursor::new(decompressed);
            let mut helper = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
            let tag_id = helper
                .get_u8()
                .map_err(|e| ReadingError::Message(e.to_string()))?;
            if tag_id == pumpkin_nbt::END_ID {
                return Ok(None);
            }
            let _name = helper
                .get_string()
                .map_err(|e| ReadingError::Message(e.to_string()))?;
            let tag = if tag_id == pumpkin_nbt::COMPOUND_ID {
                NbtTag::Compound(
                    NbtCompound::deserialize_content(&mut helper)
                        .map_err(|e| ReadingError::Message(e.to_string()))?,
                )
            } else {
                NbtTag::deserialize_data(&mut helper, tag_id)
                    .map_err(|e| ReadingError::Message(e.to_string()))?
            };
            Ok(Some(tag))
        }
    }
}

impl<'a, R: NetworkReadSliceExt<'a> + ?Sized> NetworkReadSliceExt<'a> for &mut R {
    #[inline]
    fn get_component_borrowed(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<TextComponent, ReadingError> {
        (**self).get_component_borrowed(version)
    }
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
    #[inline]
    fn get_nbt_borrowed(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<Option<NbtTag>, ReadingError> {
        (**self).get_nbt_borrowed(version)
    }
}

macro_rules! get_number_be {
    ($name:ident, $type:ty) => {
        #[inline]
        fn $name(&mut self) -> Result<$type, ReadingError> {
            let mut buf = [0u8; std::mem::size_of::<$type>()];
            self.read_exact(&mut buf)
                .map_err(|err| ReadingError::Incomplete(err.to_string().into()))?;
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
        let maximum_utf8_bytes = bound.saturating_mul(3).min(crate::MAX_PACKET_DATA_SIZE);
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

#[must_use]
pub fn json_to_nbt_tag(value: &serde_json::Value) -> NbtTag {
    match value {
        serde_json::Value::Null => NbtTag::Compound(NbtCompound::new()),
        serde_json::Value::Bool(b) => NbtTag::Byte(i8::from(*b)),
        serde_json::Value::Number(num) => num.as_i64().map_or_else(
            || num.as_f64().map_or(NbtTag::Int(0), NbtTag::Double),
            |i| {
                i8::try_from(i).map_or_else(
                    |_| {
                        i16::try_from(i).map_or_else(
                            |_| i32::try_from(i).map_or(NbtTag::Long(i), NbtTag::Int),
                            NbtTag::Short,
                        )
                    },
                    NbtTag::Byte,
                )
            },
        ),
        serde_json::Value::String(s) => NbtTag::String(s.clone().into_boxed_str()),
        serde_json::Value::Array(arr) => {
            let list: Vec<NbtTag> = arr.iter().map(json_to_nbt_tag).collect();
            NbtTag::List(list)
        }
        serde_json::Value::Object(obj) => {
            let mut compound = NbtCompound::new();
            for (k, v) in obj {
                compound.put(k, json_to_nbt_tag(v));
            }
            NbtTag::Compound(compound)
        }
    }
}

#[must_use]
pub fn nbt_tag_to_json(tag: &NbtTag) -> serde_json::Value {
    nbt_tag_to_json_ext(tag, false)
}

#[must_use]
pub fn nbt_tag_to_json_ext(tag: &NbtTag, parse_byte_as_bool: bool) -> serde_json::Value {
    match tag {
        NbtTag::End => serde_json::Value::Null,
        NbtTag::Byte(b) => {
            if parse_byte_as_bool {
                if *b == 0 {
                    serde_json::Value::Bool(false)
                } else if *b == 1 {
                    serde_json::Value::Bool(true)
                } else {
                    serde_json::Value::Number((*b).into())
                }
            } else {
                serde_json::Value::Number((*b).into())
            }
        }
        NbtTag::Short(s) => serde_json::Value::Number((*s).into()),
        NbtTag::Int(i) => serde_json::Value::Number((*i).into()),
        NbtTag::Long(l) => serde_json::Value::Number((*l).into()),
        NbtTag::Float(f) => serde_json::Number::from_f64(f64::from(*f))
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        NbtTag::Double(d) => serde_json::Number::from_f64(*d)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        NbtTag::ByteArray(bytes) => serde_json::Value::Array(
            bytes
                .iter()
                .map(|b| serde_json::Value::Number((*b).into()))
                .collect(),
        ),
        NbtTag::String(s) => serde_json::Value::String(s.to_string()),
        NbtTag::List(list) => serde_json::Value::Array(
            list.iter()
                .map(|t| nbt_tag_to_json_ext(t, parse_byte_as_bool))
                .collect(),
        ),
        NbtTag::Compound(compound) => {
            let mut map = serde_json::Map::new();
            for (k, v) in &compound.child_tags {
                map.insert(k.to_string(), nbt_tag_to_json_ext(v, parse_byte_as_bool));
            }
            serde_json::Value::Object(map)
        }
        NbtTag::IntArray(ints) => serde_json::Value::Array(
            ints.iter()
                .map(|i| serde_json::Value::Number((*i).into()))
                .collect(),
        ),
        NbtTag::LongArray(longs) => serde_json::Value::Array(
            longs
                .iter()
                .map(|l| serde_json::Value::Number((*l).into()))
                .collect(),
        ),
    }
}

pub fn write_nbt_payload(
    mut write: impl Write,
    nbt_data: &[u8],
    version: &JavaMinecraftVersion,
) -> Result<(), WritingError> {
    if *version >= JavaMinecraftVersion::V_1_8 {
        if nbt_data.is_empty() || nbt_data == [0] {
            write.write_u8(0)?;
        } else if *version < JavaMinecraftVersion::V_1_20_2 {
            if nbt_data.len() >= 3 && nbt_data[0] == 0x0A && nbt_data[1] == 0 && nbt_data[2] == 0 {
                write.write_all(nbt_data).map_err(WritingError::IoError)?;
            } else if nbt_data[0] == 0x0A {
                write.write_u8(0x0A)?;
                write.write_u16_be(0)?;
                write
                    .write_all(&nbt_data[1..])
                    .map_err(WritingError::IoError)?;
            } else {
                write.write_all(nbt_data).map_err(WritingError::IoError)?;
            }
        } else {
            write.write_all(nbt_data).map_err(WritingError::IoError)?;
        }
    } else {
        if nbt_data.is_empty() || nbt_data == [0] {
            write.write_i16_be(-1)?;
        } else {
            let mut named_bytes = Vec::with_capacity(nbt_data.len() + 2);
            if nbt_data.len() >= 3 && nbt_data[0] == 0x0A && nbt_data[1] == 0 && nbt_data[2] == 0 {
                named_bytes.extend_from_slice(nbt_data);
            } else if nbt_data[0] == 0x0A {
                named_bytes.push(0x0A);
                named_bytes.extend_from_slice(&[0x00, 0x00]);
                named_bytes.extend_from_slice(&nbt_data[1..]);
            } else {
                named_bytes.extend_from_slice(nbt_data);
            }

            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder
                .write_all(&named_bytes)
                .map_err(WritingError::IoError)?;
            let compressed = encoder.finish().map_err(WritingError::IoError)?;

            write.write_i16_be(compressed.len() as i16)?;
            write
                .write_all(&compressed)
                .map_err(WritingError::IoError)?;
        }
    }
    Ok(())
}

pub fn read_nbt_payload(
    bytebuf: &mut &[u8],
    version: &JavaMinecraftVersion,
) -> Result<Box<[u8]>, ReadingError> {
    if *version >= JavaMinecraftVersion::V_1_8 {
        if bytebuf.is_empty() || bytebuf[0] == 0 {
            if !bytebuf.is_empty() {
                let _ = bytebuf.get_u8()?;
            }
            Ok(Box::new([]))
        } else if *version < JavaMinecraftVersion::V_1_20_2 {
            let all = bytebuf.to_vec();
            *bytebuf = &[];
            if all.len() >= 3 && all[0] == 0x0A && all[1] == 0 && all[2] == 0 {
                let mut unnamed = Vec::with_capacity(all.len() - 2);
                unnamed.push(0x0A);
                unnamed.extend_from_slice(&all[3..]);
                Ok(unnamed.into_boxed_slice())
            } else {
                Ok(all.into_boxed_slice())
            }
        } else {
            let all = bytebuf.to_vec().into_boxed_slice();
            *bytebuf = &[];
            Ok(all)
        }
    } else {
        let length = bytebuf.get_i16_be()?;
        if length <= 0 {
            Ok(Box::new([]))
        } else {
            if bytebuf.len() < length as usize {
                return Err(ReadingError::Incomplete(
                    "Not enough bytes for compressed NBT".into(),
                ));
            }
            let compressed = &bytebuf[..length as usize];
            *bytebuf = &bytebuf[length as usize..];
            let mut decoder = flate2::read::GzDecoder::new(compressed);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|e| ReadingError::Message(e.to_string()))?;
            if decompressed.len() >= 3
                && decompressed[0] == 0x0A
                && decompressed[1] == 0
                && decompressed[2] == 0
            {
                let mut unnamed = Vec::with_capacity(decompressed.len() - 2);
                unnamed.push(0x0A);
                unnamed.extend_from_slice(&decompressed[3..]);
                Ok(unnamed.into_boxed_slice())
            } else {
                Ok(decompressed.into_boxed_slice())
            }
        }
    }
}

#[inline]
pub fn read_remaining_bytes(
    read: &mut (impl Read + ?Sized),
    bound: usize,
) -> Result<Box<[u8]>, ReadingError> {
    let mut return_buf = Vec::with_capacity(bound.min(1024));

    // Take one extra byte to check for exceeding bound
    read.take(bound as u64 + 1)
        .read_to_end(&mut return_buf)
        .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

    if return_buf.len() > bound {
        return Err(ReadingError::TooLarge("Read remaining too long".into()));
    }

    Ok(return_buf.into_boxed_slice())
}

pub trait NetworkWriteExt {
    fn write_component(
        &mut self,
        component: &TextComponent,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version < JavaMinecraftVersion::V_1_20_3 {
            let json = component.to_json_for_version(version);
            let max_len = if *version >= JavaMinecraftVersion::V_1_13 {
                262144
            } else {
                32767
            };
            self.write_string_bounded(&json, max_len)
        } else {
            self.write_slice(&component.encode_for_version(version))
        }
    }

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
    fn write_block_pos(
        &mut self,
        pos: &BlockPos,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError>;

    #[inline]
    fn write_container_id(
        &mut self,
        container_id: &VarInt,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_21_2 {
            self.write_var_int(container_id)
        } else {
            self.write_u8(container_id.0 as u8)
        }
    }

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
    fn write_nbt_with_version(
        &mut self,
        data: Option<&NbtTag>,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError>;

    #[inline]
    fn write_compound_nbt_with_version(
        &mut self,
        data: Option<&NbtCompound>,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let tag = data.map(|c| NbtTag::Compound(c.clone()));
        self.write_nbt_with_version(tag.as_ref(), version)
    }

    #[inline]
    fn write_nbt(&mut self, data: NbtTag) -> Result<(), WritingError> {
        self.write_nbt_with_version(Some(&data), &JavaMinecraftVersion::V_26_2)
    }
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
        let bytes_to_copy = std::cmp::min(bit_set.len(), new_length);

        self.write_slice(&bit_set[..bytes_to_copy])?;

        if new_length > bytes_to_copy {
            const ZEROES: [u8; 64] = [0u8; 64];
            let padding = new_length - bytes_to_copy;
            let mut remaining = padding;
            while remaining > 0 {
                let chunk = remaining.min(ZEROES.len());
                self.write_slice(&ZEROES[..chunk])?;
                remaining -= chunk;
            }
        }

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

    fn write_block_pos(
        &mut self,
        pos: &BlockPos,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        self.write_i64_be(pos.as_long_for_version(version))
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
        self.write_var_int(&VarInt(list.len() as i32))?;

        for data in list {
            writer(self, data)?;
        }

        Ok(())
    }

    fn write_nbt_with_version(
        &mut self,
        data: Option<&NbtTag>,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_8 {
            if let Some(tag) = data
                && !matches!(tag, NbtTag::End)
            {
                if *version < JavaMinecraftVersion::V_1_20_2 {
                    self.write_u8(tag.get_type_id())?;
                    self.write_u16_be(0)?; // empty root name
                    let mut write_adaptor = NbtWriteHelperJava::new(self);
                    tag.clone()
                        .serialize_data(&mut write_adaptor)
                        .map_err(|e| WritingError::Message(e.to_string()))?;
                } else {
                    let mut write_adaptor = NbtWriteHelperJava::new(self);
                    tag.clone()
                        .serialize(&mut write_adaptor)
                        .map_err(|e| WritingError::Message(e.to_string()))?;
                }
            } else {
                self.write_u8(pumpkin_nbt::END_ID)?;
            }
        } else {
            // <= 1.7.10
            if let Some(tag) = data
                && !matches!(tag, NbtTag::End)
            {
                let mut buf = Vec::new();
                buf.push(tag.get_type_id());
                buf.extend_from_slice(&[0x00, 0x00]); // empty root name
                let mut write_adaptor = NbtWriteHelperJava::new(&mut buf);
                tag.clone()
                    .serialize_data(&mut write_adaptor)
                    .map_err(|e| WritingError::Message(e.to_string()))?;

                let mut encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(&buf).map_err(WritingError::IoError)?;
                let compressed = encoder.finish().map_err(WritingError::IoError)?;

                self.write_i16_be(compressed.len() as i16)?;
                self.write_slice(&compressed)?;
            } else {
                self.write_i16_be(-1)?;
            }
        }
        Ok(())
    }
}

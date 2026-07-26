use std::{
    fmt::Display,
    io::{self, Write},
    ops::Deref,
};

use bytes::Bytes;
use deserializer::NbtReadHelper;
use serde::{de, ser};
use serializer::{NbtWriteHelper, NbtWriteHelperBedrock, NbtWriteHelperJava};
use tag::NbtTag;
use thiserror::Error;

pub mod compound;
pub mod deserializer;
pub mod nbt_compress;
pub mod nbt_ops;
pub mod serializer;
pub mod tag;

pub use compound::NbtCompound;
pub use deserializer::{
    from_bytes, from_bytes_bedrock, from_bytes_unnamed, from_slice, from_slice_bedrock,
    from_slice_unnamed,
};
pub use serializer::{to_bytes, to_bytes_named, to_bytes_unnamed};

// This NBT crate is inspired from CrabNBT

pub const END_ID: u8 = 0x00;
pub const BYTE_ID: u8 = 0x01;
pub const SHORT_ID: u8 = 0x02;
pub const INT_ID: u8 = 0x03;
pub const LONG_ID: u8 = 0x04;
pub const FLOAT_ID: u8 = 0x05;
pub const DOUBLE_ID: u8 = 0x06;
pub const BYTE_ARRAY_ID: u8 = 0x07;
pub const STRING_ID: u8 = 0x08;
pub const LIST_ID: u8 = 0x09;
pub const COMPOUND_ID: u8 = 0x0A;
pub const INT_ARRAY_ID: u8 = 0x0B;
pub const LONG_ARRAY_ID: u8 = 0x0C;

pub const MAX_ARRAY_LENGTH: usize = 2_000_000;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The root tag of the NBT file is not a compound tag. Received tag id: {0}")]
    NoRootCompound(u8),
    #[error("Encountered an unknown NBT tag id: {0}.")]
    UnknownTagId(u8),
    #[error("Failed to Cesu 8 Decode")]
    Cesu8DecodingError,
    #[error("Failed to UTF-8 Decode")]
    Utf8DecodingError,
    #[error("Serde error: {0}")]
    SerdeError(String),
    #[error("NBT doesn't support this type: {0}")]
    UnsupportedType(String),
    #[error("NBT reading was cut short: {0}")]
    Incomplete(io::Error),
    #[error("Negative list length: {0}")]
    NegativeLength(i32),
    #[error("Length too large: {0}")]
    LargeLength(usize),
    #[error("Failed to decode varint - value too large")]
    VarIntTooLarge,
    #[error("Failed to decode varlong - value too large")]
    VarLongTooLarge,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::SerdeError(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::SerdeError(msg.to_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Nbt {
    pub name: String,
    pub root_tag: NbtCompound,
}

impl Nbt {
    #[must_use]
    pub const fn new(name: String, tag: NbtCompound) -> Self {
        Self {
            name,
            root_tag: tag,
        }
    }

    pub fn read<'a, R: NbtReadHelper<'a>>(reader: &mut R) -> Result<Self, Error> {
        let tag_type_id = reader.get_u8()?;

        if tag_type_id != COMPOUND_ID {
            return Err(Error::NoRootCompound(tag_type_id));
        }

        Ok(Self {
            name: reader.get_string()?.into_owned(),
            root_tag: NbtCompound::deserialize_content(reader)?,
        })
    }

    /// Reads an NBT tag that doesn't contain the name of the root `Compound`.
    pub fn read_unnamed<'a, R: NbtReadHelper<'a>>(reader: &mut R) -> Result<Self, Error> {
        let tag_type_id = reader.get_u8()?;

        if tag_type_id != COMPOUND_ID {
            return Err(Error::NoRootCompound(tag_type_id));
        }

        Ok(Self {
            name: String::new(),
            root_tag: NbtCompound::deserialize_content(reader)?,
        })
    }

    #[must_use]
    pub fn write(self) -> Bytes {
        let mut bytes = Vec::new();
        let mut writer = NbtWriteHelperJava::new(&mut bytes);
        writer.write_u8(COMPOUND_ID).unwrap();
        NbtTag::String(self.name.into())
            .serialize_data(&mut writer)
            .unwrap();
        self.root_tag.serialize_content(&mut writer).unwrap();

        bytes.into()
    }

    #[must_use]
    pub fn write_bedrock(self) -> Bytes {
        let mut bytes = Vec::new();
        let mut writer = NbtWriteHelperBedrock::new(&mut bytes);
        writer.write_u8(COMPOUND_ID).unwrap();
        NbtTag::String(self.name.into())
            .serialize_data(&mut writer)
            .unwrap();
        self.root_tag.serialize_content(&mut writer).unwrap();

        bytes.into()
    }

    pub fn write_to_writer<W: Write>(self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.write())?;
        Ok(())
    }

    pub fn write_to_writer_bedrock<W: Write>(self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.write_bedrock())?;
        Ok(())
    }

    /// Writes an NBT tag without a root `Compound` name.
    #[must_use]
    pub fn write_unnamed(self) -> Bytes {
        let mut bytes = Vec::new();
        let mut writer = NbtWriteHelperJava::new(&mut bytes);

        writer.write_u8(COMPOUND_ID).unwrap();
        self.root_tag.serialize_content(&mut writer).unwrap();

        bytes.into()
    }

    pub fn write_unnamed_to_writer<W: Write>(self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.write_unnamed())?;
        Ok(())
    }
}

impl Deref for Nbt {
    type Target = NbtCompound;

    fn deref(&self) -> &Self::Target {
        &self.root_tag
    }
}

impl From<NbtCompound> for Nbt {
    fn from(value: NbtCompound) -> Self {
        Self::new(String::new(), value)
    }
}

impl<T> AsRef<T> for Nbt
where
    T: ?Sized,
    <Self as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl AsMut<NbtCompound> for Nbt {
    fn as_mut(&mut self) -> &mut NbtCompound {
        &mut self.root_tag
    }
}

// TODO: This is a bit hacky
pub(crate) const NBT_ARRAY_TAG: &str = "__nbt_array";
pub(crate) const NBT_INT_ARRAY_TAG: &str = "__nbt_int_array";
pub(crate) const NBT_LONG_ARRAY_TAG: &str = "__nbt_long_array";
pub(crate) const NBT_BYTE_ARRAY_TAG: &str = "__nbt_byte_array";

macro_rules! impl_array {
    ($name:ident, $variant:expr) => {
        pub fn $name<T: serde::Serialize, S: serde::Serializer>(
            input: T,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            serializer.serialize_newtype_variant(NBT_ARRAY_TAG, 0, $variant, &input)
        }
    };
}

impl_array!(nbt_int_array, NBT_INT_ARRAY_TAG);
impl_array!(nbt_long_array, NBT_LONG_ARRAY_TAG);
impl_array!(nbt_byte_array, NBT_BYTE_ARRAY_TAG);

#[cfg(test)]
mod test;

#[cfg(test)]
mod api_test {
    use std::io::Cursor;

    use crate::Nbt;
    use crate::compound::NbtCompound;
    use crate::deserializer::{NbtReadHelperBedrock, NbtReadHelperJava};

    #[test]
    fn named_write_read_roundtrip() {
        let mut compound = NbtCompound::new();
        compound.put_int("int", 42);
        compound.put_string("string", "pumpkin".to_string());
        compound.put_long("long", -7);

        let bytes = Nbt::new("root".to_string(), compound.clone()).write();
        let mut reader = NbtReadHelperJava::new(Cursor::new(bytes.to_vec()));
        let read = Nbt::read(&mut reader).unwrap();

        assert_eq!(read.name, "root");
        assert_eq!(read.root_tag, compound);
    }

    #[test]
    fn unnamed_write_read_roundtrip() {
        let mut compound = NbtCompound::new();
        compound.put_short("short", 12);

        let bytes = Nbt::new("ignored".to_string(), compound.clone()).write_unnamed();
        let mut reader = NbtReadHelperJava::new(Cursor::new(bytes.to_vec()));
        let read = Nbt::read_unnamed(&mut reader).unwrap();

        assert_eq!(read.name, "");
        assert_eq!(read.root_tag, compound);
    }

    #[test]
    fn bedrock_write_read_roundtrip() {
        let mut compound = NbtCompound::new();
        compound.put_int("int", 1234);
        compound.put_string("string", "bedrock".to_string());

        let bytes = Nbt::new("root".to_string(), compound.clone()).write_bedrock();
        let mut reader = NbtReadHelperBedrock::new(Cursor::new(bytes.to_vec()));
        let read = Nbt::read(&mut reader).unwrap();

        assert_eq!(read.name, "root");
        assert_eq!(read.root_tag, compound);
    }

    #[test]
    fn named_write_known_bytes() {
        let mut compound = NbtCompound::new();
        compound.put_byte("b", 1);

        let bytes = Nbt::new("a".to_string(), compound).write();
        let expected_bytes = [
            0x0A, // Compound tag
            0x00, 0x01, // Root name length
            0x61, // "a"
            0x01, // Byte tag
            0x00, 0x01, // Key length
            0x62, // "b"
            0x01, // 1
            0x00, // End tag
        ];
        assert_eq!(bytes.to_vec(), expected_bytes);
    }

    #[test]
    fn nbt_public_paths_reachable() {
        let make: fn(String, NbtCompound) -> Nbt = Nbt::new;
        let _: fn(Nbt) -> bytes::Bytes = Nbt::write;
        let _: fn(Nbt) -> bytes::Bytes = Nbt::write_unnamed;
        let _: fn(Nbt) -> bytes::Bytes = Nbt::write_bedrock;
        let nbt = make("n".to_string(), NbtCompound::new());
        assert_eq!(nbt.name, "n");
    }
}

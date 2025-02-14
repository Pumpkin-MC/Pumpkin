use std::{
    fmt::Display,
    io::{self, Read, Write},
    ops::Deref,
};

use bytes::Bytes;
use compound::NbtCompound;
use deserializer::ReadAdaptor;
use serde::{de, ser};
use serde::{Deserialize, Deserializer};
use serializer::WriteAdaptor;
use tag::NbtTag;
use thiserror::Error;

pub mod compound;
pub mod deserializer;
pub mod serializer;
pub mod tag;

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

#[derive(Error, Debug)]
pub enum Error {
    #[error("The root tag of the NBT file is not a compound tag. Received tag id: {0}")]
    NoRootCompound(u8),
    #[error("Encountered an unknown NBT tag id {0}.")]
    UnknownTagId(u8),
    #[error("Failed to Cesu 8 Decode")]
    Cesu8DecodingError,
    #[error("Serde error: {0}")]
    SerdeError(String),
    #[error("NBT doesn't support this type {0}")]
    UnsupportedType(String),
    #[error("NBT reading was cut short {0}")]
    Incomplete(io::Error),
    #[error("Negative list length {0}")]
    NegativeLength(i32),
    #[error("Length too large {0}")]
    LargeLength(usize),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::SerdeError(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::SerdeError(msg.to_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Nbt {
    pub name: String,
    pub root_tag: NbtCompound,
}

impl Nbt {
    pub fn new(name: String, tag: NbtCompound) -> Self {
        Nbt {
            name,
            root_tag: tag,
        }
    }

    pub fn read<R>(reader: &mut ReadAdaptor<R>) -> Result<Nbt, Error>
    where
        R: Read,
    {
        let tag_type_id = reader.get_u8_be()?;

        if tag_type_id != COMPOUND_ID {
            return Err(Error::NoRootCompound(tag_type_id));
        }

        Ok(Nbt {
            name: get_nbt_string(reader)?,
            root_tag: NbtCompound::deserialize_content(reader)?,
        })
    }

    /// Reads NBT tag, that doesn't contain the name of root compound.
    pub fn read_unnamed<R>(reader: &mut ReadAdaptor<R>) -> Result<Nbt, Error>
    where
        R: Read,
    {
        let tag_type_id = reader.get_u8_be()?;

        if tag_type_id != COMPOUND_ID {
            return Err(Error::NoRootCompound(tag_type_id));
        }

        Ok(Nbt {
            name: String::new(),
            root_tag: NbtCompound::deserialize_content(reader)?,
        })
    }

    pub fn write(&self) -> Bytes {
        let mut bytes = Vec::new();
        let mut writer = WriteAdaptor::new(&mut bytes);
        writer.write_u8_be(COMPOUND_ID).unwrap();
        NbtTag::String(self.name.to_string())
            .serialize_data(&mut writer)
            .unwrap();
        self.root_tag.serialize_content(&mut writer).unwrap();

        bytes.into()
    }

    pub fn write_to_writer<W: Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.write())?;
        Ok(())
    }

    /// Writes NBT tag, without name of root compound.
    pub fn write_unnamed(&self) -> Bytes {
        let mut bytes = Vec::new();
        let mut writer = WriteAdaptor::new(&mut bytes);

        writer.write_u8_be(COMPOUND_ID).unwrap();
        self.root_tag.serialize_content(&mut writer).unwrap();

        bytes.into()
    }

    pub fn write_unnamed_to_writer<W: Write>(&self, mut writer: W) -> Result<(), io::Error> {
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
        Nbt::new(String::new(), value)
    }
}

impl<T> AsRef<T> for Nbt
where
    T: ?Sized,
    <Nbt as Deref>::Target: AsRef<T>,
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

pub fn get_nbt_string<R: Read>(bytes: &mut ReadAdaptor<R>) -> Result<String, Error> {
    let len = bytes.get_u16_be()? as usize;
    let string_bytes = bytes.read_boxed_slice(len)?;
    let string = cesu8::from_java_cesu8(&string_bytes).map_err(|_| Error::Cesu8DecodingError)?;
    Ok(string.to_string())
}

macro_rules! impl_array {
    ($name:ident, $variant:expr) => {
        pub struct $name;

        impl $name {
            pub fn serialize<T, S>(input: T, serializer: S) -> Result<S::Ok, S::Error>
            where
                T: serde::Serialize,
                S: serde::Serializer,
            {
                serializer.serialize_newtype_variant("nbt_array", 0, $variant, &input)
            }

            pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
            where
                T: Deserialize<'de>,
                D: Deserializer<'de>,
            {
                T::deserialize(deserializer)
            }
        }
    };
}

impl_array!(IntArray, "int");
impl_array!(LongArray, "long");
impl_array!(BytesArray, "byte");

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use flate2::read::GzDecoder;
    use serde::{Deserialize, Serialize};

    use pumpkin_world::world_info::{DataPacks, LevelData, WorldGenSettings, WorldVersion};

    use crate::deserializer::from_bytes;
    use crate::serializer::to_bytes;
    use crate::BytesArray;
    use crate::IntArray;
    use crate::LongArray;
    use crate::{deserializer::from_bytes_unnamed, serializer::to_bytes_unnamed};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Test {
        byte: i8,
        short: i16,
        int: i32,
        long: i64,
        float: f32,
        string: String,
    }

    #[test]
    fn test_simple_ser_de_unnamed() {
        let test = Test {
            byte: 123,
            short: 1342,
            int: 4313,
            long: 34,
            float: 1.00,
            string: "Hello test".to_string(),
        };

        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let recreated_struct: Test = from_bytes_unnamed(&bytes[..]).unwrap();

        assert_eq!(test, recreated_struct);
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestArray {
        #[serde(with = "BytesArray")]
        byte_array: Vec<u8>,
        #[serde(with = "IntArray")]
        int_array: Vec<i32>,
        #[serde(with = "LongArray")]
        long_array: Vec<i64>,
    }

    #[test]
    fn test_simple_ser_de_array() {
        let test = TestArray {
            byte_array: vec![0, 3, 2],
            int_array: vec![13, 1321, 2],
            long_array: vec![1, 0, 200301, 1],
        };

        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let recreated_struct: TestArray = from_bytes_unnamed(&bytes[..]).unwrap();

        assert_eq!(test, recreated_struct);
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct LevelDat {
        // This tag contains all the level data.
        #[serde(rename = "Data")]
        data: LevelData,
    }

    static LEVEL_DAT: LazyLock<LevelDat> = LazyLock::new(|| LevelDat {
        data: LevelData {
            allow_commands: true,
            border_center_x: 0.0,
            border_center_z: 0.0,
            border_damage_per_block: 0.2,
            border_size: 59_999_968.0,
            border_safe_zone: 5.0,
            border_size_lerp_target: 59_999_968.0,
            border_size_lerp_time: 0,
            border_warning_blocks: 5.0,
            border_warning_time: 15.0,
            clear_weather_time: 0,
            data_packs: DataPacks {
                disabled: vec![
                    "minecart_improvements".to_string(),
                    "redstone_experiments".to_string(),
                    "trade_rebalance".to_string(),
                ],
                enabled: vec!["vanilla".to_string()],
            },
            data_version: 4189,
            day_time: 1727,
            difficulty: 2,
            difficulty_locked: false,
            world_gen_settings: WorldGenSettings { seed: 1 },
            last_played: 1733847709327,
            level_name: "New World".to_string(),
            spawn_x: 160,
            spawn_y: 70,
            spawn_z: 160,
            spawn_angle: 0.0,
            nbt_version: 19133,
            version: WorldVersion {
                name: "1.21.4".to_string(),
                id: 4189,
                snapshot: false,
                series: "main".to_string(),
            },
        },
    });

    // TODO: More robust tests

    #[test]
    fn test_deserialize_level_dat() {
        let raw_compressed_nbt = include_bytes!("../assets/level.dat");
        assert!(!raw_compressed_nbt.is_empty());

        let decoder = GzDecoder::new(&raw_compressed_nbt[..]);
        let level_dat: LevelDat = from_bytes(decoder).expect("Failed to decode from file");

        assert_eq!(level_dat, *LEVEL_DAT);
    }

    #[test]
    fn test_serialize_level_dat() {
        let mut serialized = Vec::new();
        to_bytes(&*LEVEL_DAT, &mut serialized).expect("Failed to encode to bytes");

        assert!(!serialized.is_empty());

        let level_dat_again: LevelDat =
            from_bytes(&serialized[..]).expect("Failed to decode from bytes");

        assert_eq!(level_dat_again, *LEVEL_DAT);
    }
}

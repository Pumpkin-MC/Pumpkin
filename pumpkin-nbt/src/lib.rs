use std::{
    fmt::Display,
    io::{self, Read, Write},
    ops::Deref,
};

use bytes::Bytes;
use compound::NbtCompound;
use deserializer::ReadAdaptor;
use serde::{de, ser};
use serializer::WriteAdaptor;
use tag::NbtTag;
use thiserror::Error;

pub mod compound;
pub mod deserializer;
pub mod serializer;
pub mod tag;

pub use deserializer::{from_bytes, from_bytes_unnamed};
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

pub(crate) const NBT_ARRAY_TAG: &str = "__nbt_array";
pub(crate) const NBT_INT_ARRAY_TAG: &str = "__nbt_int_array";
pub(crate) const NBT_LONG_ARRAY_TAG: &str = "__nbt_long_array";
pub(crate) const NBT_BYTE_ARRAY_TAG: &str = "__nbt_byte_array";

macro_rules! impl_array {
    ($name:ident, $variant:expr) => {
        pub fn $name<T, S>(input: T, serializer: S) -> Result<S::Ok, S::Error>
        where
            T: serde::Serialize,
            S: serde::Serializer,
        {
            serializer.serialize_newtype_variant(NBT_ARRAY_TAG, 0, $variant, &input)
        }
    };
}

impl_array!(nbt_int_array, NBT_INT_ARRAY_TAG);
impl_array!(nbt_long_array, NBT_LONG_ARRAY_TAG);
impl_array!(nbt_byte_array, NBT_BYTE_ARRAY_TAG);

#[cfg(test)]
mod test {

    use crate::Error;
    use crate::deserializer::from_bytes;
    use crate::nbt_byte_array;
    use crate::nbt_int_array;
    use crate::nbt_long_array;
    use crate::serializer::to_bytes;
    use crate::serializer::to_bytes_named;
    use crate::{deserializer::from_bytes_unnamed, serializer::to_bytes_unnamed};
    use serde::{Deserialize, Serialize};

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
        #[serde(serialize_with = "nbt_byte_array")]
        byte_array: Vec<u8>,
        #[serde(serialize_with = "nbt_int_array")]
        int_array: Vec<i32>,
        #[serde(serialize_with = "nbt_long_array")]
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

    #[test]
    fn test_simple_ser_de_named() {
        let name = String::from("Test");
        let test = Test {
            byte: 123,
            short: 1342,
            int: 4313,
            long: 34,
            float: 1.00,
            string: "Hello test".to_string(),
        };

        let mut bytes = Vec::new();
        to_bytes_named(&test, name, &mut bytes).unwrap();
        let recreated_struct: Test = from_bytes(&bytes[..]).unwrap();

        assert_eq!(test, recreated_struct);
    }

    #[test]
    fn test_simple_ser_de_array_named() {
        let name = String::from("Test");
        let test = TestArray {
            byte_array: vec![0, 3, 2],
            int_array: vec![13, 1321, 2],
            long_array: vec![1, 0, 200301, 1],
        };

        let mut bytes = Vec::new();
        to_bytes_named(&test, name, &mut bytes).unwrap();
        let recreated_struct: TestArray = from_bytes(&bytes[..]).unwrap();

        assert_eq!(test, recreated_struct);
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Egg {
        food: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Breakfast {
        food: Egg,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestList {
        option: Option<Egg>,
        nested_compound: Breakfast,
        compounds: Vec<Test>,
        list_string: Vec<String>,
        empty: Vec<Test>,
    }

    #[test]
    fn test_list() {
        let test1 = Test {
            byte: 123,
            short: 1342,
            int: 4313,
            long: 34,
            float: 1.00,
            string: "Hello test".to_string(),
        };

        let test2 = Test {
            byte: 13,
            short: 342,
            int: -4313,
            long: -132334,
            float: -69.420,
            string: "Hello compounds".to_string(),
        };

        let list_compound = TestList {
            option: Some(Egg {
                food: "Skibid".to_string(),
            }),
            nested_compound: Breakfast {
                food: Egg {
                    food: "Over easy".to_string(),
                },
            },
            compounds: vec![test1, test2],
            list_string: vec!["".to_string(), "abcbcbcbbc".to_string()],
            empty: vec![],
        };

        let mut bytes = Vec::new();
        to_bytes_unnamed(&list_compound, &mut bytes).unwrap();
        let recreated_struct: TestList = from_bytes_unnamed(&bytes[..]).unwrap();
        assert_eq!(list_compound, recreated_struct);
    }

    #[test]
    fn test_list_named() {
        let test1 = Test {
            byte: 123,
            short: 1342,
            int: 4313,
            long: 34,
            float: 1.00,
            string: "Hello test".to_string(),
        };

        let test2 = Test {
            byte: 13,
            short: 342,
            int: -4313,
            long: -132334,
            float: -69.420,
            string: "Hello compounds".to_string(),
        };

        let list_compound = TestList {
            option: None,
            nested_compound: Breakfast {
                food: Egg {
                    food: "Over easy".to_string(),
                },
            },
            compounds: vec![test1, test2],
            list_string: vec!["".to_string(), "abcbcbcbbc".to_string()],
            empty: vec![],
        };

        let mut bytes = Vec::new();
        to_bytes_named(&list_compound, "a".to_string(), &mut bytes).unwrap();
        let recreated_struct: TestList = from_bytes(&bytes[..]).unwrap();
        assert_eq!(list_compound, recreated_struct);
    }

    #[test]
    fn test_nbt_arrays() {
        #[derive(Serialize)]
        struct Tagged {
            #[serde(serialize_with = "nbt_long_array")]
            l: [i64; 1],
            #[serde(serialize_with = "nbt_int_array")]
            i: [i32; 1],
            #[serde(serialize_with = "nbt_byte_array")]
            b: [u8; 1],
        }

        let value = Tagged {
            l: [0],
            i: [0],
            b: [0],
        };
        let expected_bytes = [
            0x0A, // Component Tag
            0x00, 0x00, // Empty root name
            0x0C, // Long Array Type
            0x00, 0x01, // Key length
            0x6C, // Key (l)
            0x00, 0x00, 0x00, 0x01, // Array Length
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Value(s)
            0x0B, // Int Array Tag
            0x00, 0x01, // Key length
            0x69, // Key (i)
            0x00, 0x00, 0x00, 0x01, // Array Length
            0x00, 0x00, 0x00, 0x00, // Value(s)
            0x07, // Byte Array Tag
            0x00, 0x01, // Key length
            0x62, // Key (b)
            0x00, 0x00, 0x00, 0x01, // Array Length
            0x00, // Value(s)
            0x00, // End Tag
        ];

        let mut bytes = Vec::new();
        to_bytes(&value, &mut bytes).unwrap();
        assert_eq!(bytes, expected_bytes);

        #[derive(Serialize)]
        struct NotTagged {
            l: [i64; 1],
            i: [i32; 1],
            b: [u8; 1],
        }

        let value = NotTagged {
            l: [0],
            i: [0],
            b: [0],
        };
        let expected_bytes = [
            0x0A, // Component Tag
            0x00, 0x00, // Empty root name
            0x09, // List Tag
            0x00, 0x01, // Key length
            0x6C, // Key (l)
            0x04, // Array Type
            0x00, 0x00, 0x00, 0x01, // Array Length
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Value(s)
            0x09, // List Tag
            0x00, 0x01, // Key length
            0x69, // Key (i)
            0x03, // Array Type
            0x00, 0x00, 0x00, 0x01, // Array Length
            0x00, 0x00, 0x00, 0x00, // Value(s)
            0x09, // List Tag
            0x00, 0x01, // Key length
            0x62, // Key (b)
            0x01, // Array Type
            0x00, 0x00, 0x00, 0x01, // Array Length
            0x00, // Value(s)
            0x00, // End Tag
        ];

        let mut bytes = Vec::new();
        to_bytes(&value, &mut bytes).unwrap();
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_tuple_fail() {
        #[derive(Serialize)]
        struct BadData {
            x: (i32, i64),
        }

        let value = BadData { x: (0, 0) };
        let mut bytes = Vec::new();
        let err = to_bytes(&value, &mut bytes);

        match err {
            Err(Error::SerdeError(_)) => (),
            _ => panic!("Expected to fail serialization!"),
        };
    }

    #[test]
    fn test_tuple_ok() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct GoodData {
            x: (i32, i32),
        }

        let value = GoodData { x: (1, 2) };
        let mut bytes = Vec::new();
        to_bytes(&value, &mut bytes).unwrap();

        let reconstructed = from_bytes(&bytes[..]).unwrap();
        assert_eq!(value, reconstructed);
    }

    // TODO: More robust tests
}

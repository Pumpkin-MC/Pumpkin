use std::{
    fmt::Display,
    io::{self, Read, Seek, Write},
    ops::Deref,
};

use bytes::Bytes;
use compound::NbtCompound;
use deserializer::NbtReadHelper;
use serde::{de, ser};
use serializer::WriteAdaptor;
use tag::NbtTag;
use thiserror::Error;

pub mod anvil;
pub mod compound;
pub mod deserializer;
pub mod nbt_compress;
pub mod player_data;
pub mod serializer;
pub mod snbt;
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
    #[error("Encountered an unknown NBT tag id: {0}.")]
    UnknownTagId(u8),
    #[error("Failed to Cesu 8 Decode")]
    Cesu8DecodingError,
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

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
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

    pub fn read<R: Read + Seek>(reader: &mut NbtReadHelper<R>) -> Result<Self, Error> {
        let tag_type_id = reader.get_u8_be()?;

        if tag_type_id != COMPOUND_ID {
            return Err(Error::NoRootCompound(tag_type_id));
        }

        Ok(Self {
            name: get_nbt_string(reader)?,
            root_tag: NbtCompound::deserialize_content(reader)?,
        })
    }

    /// Reads an NBT tag that doesn't contain the name of the root `Compound`.
    pub fn read_unnamed<R: Read + Seek>(reader: &mut NbtReadHelper<R>) -> Result<Self, Error> {
        let tag_type_id = reader.get_u8_be()?;

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
        let mut writer = WriteAdaptor::new(&mut bytes);
        writer.write_u8_be(COMPOUND_ID).unwrap();
        NbtTag::String(self.name)
            .serialize_data(&mut writer)
            .unwrap();
        self.root_tag.serialize_content(&mut writer).unwrap();

        bytes.into()
    }

    pub fn write_to_writer<W: Write>(self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.write())?;
        Ok(())
    }

    /// Writes an NBT tag without a root `Compound` name.
    #[must_use]
    pub fn write_unnamed(self) -> Bytes {
        let mut bytes = Vec::new();
        let mut writer = WriteAdaptor::new(&mut bytes);

        writer.write_u8_be(COMPOUND_ID).unwrap();
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

pub fn get_nbt_string<R: Read + Seek>(bytes: &mut NbtReadHelper<R>) -> Result<String, Error> {
    let len = bytes.get_u16_be()? as usize;
    let string_bytes = bytes.read_boxed_slice(len)?;
    let string = cesu8::from_java_cesu8(&string_bytes).map_err(|_| Error::Cesu8DecodingError)?;
    Ok(string.into_owned())
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
mod test {

    use std::io::Cursor;

    use crate::Error;
    use crate::Nbt;
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
    fn simple_ser_de_unnamed() {
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
        let recreated_struct: Test = from_bytes_unnamed(Cursor::new(bytes)).unwrap();

        assert_eq!(test, recreated_struct);
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[expect(clippy::struct_field_names)]
    struct TestArray {
        #[serde(serialize_with = "nbt_byte_array")]
        byte_array: Vec<u8>,
        #[serde(serialize_with = "nbt_int_array")]
        int_array: Vec<i32>,
        #[serde(serialize_with = "nbt_long_array")]
        long_array: Vec<i64>,
    }

    #[test]
    fn simple_ser_de_array() {
        let test = TestArray {
            byte_array: vec![0, 3, 2],
            int_array: vec![13, 1321, 2],
            long_array: vec![1, 0, 200301, 1],
        };

        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let recreated_struct: TestArray = from_bytes_unnamed(Cursor::new(bytes)).unwrap();

        assert_eq!(test, recreated_struct);
    }

    #[test]
    fn simple_ser_de_named() {
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
        let recreated_struct: Test = from_bytes(Cursor::new(bytes)).unwrap();

        assert_eq!(test, recreated_struct);
    }

    #[test]
    fn simple_ser_de_array_named() {
        let name = String::from("Test");
        let test = TestArray {
            byte_array: vec![0, 3, 2],
            int_array: vec![13, 1321, 2],
            long_array: vec![1, 0, 200301, 1],
        };

        let mut bytes = Vec::new();
        to_bytes_named(&test, name, &mut bytes).unwrap();
        let recreated_struct: TestArray = from_bytes(Cursor::new(bytes)).unwrap();

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
    fn list() {
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
            list_string: vec![String::new(), "abcbcbcbbc".to_string()],
            empty: vec![],
        };

        let mut bytes = Vec::new();
        to_bytes_unnamed(&list_compound, &mut bytes).unwrap();
        let recreated_struct: TestList = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert_eq!(list_compound, recreated_struct);
    }

    #[test]
    fn list_named() {
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
            list_string: vec![String::new(), "abcbcbcbbc".to_string()],
            empty: vec![],
        };

        let mut bytes = Vec::new();
        to_bytes_named(&list_compound, "a".to_string(), &mut bytes).unwrap();
        let recreated_struct: TestList = from_bytes(Cursor::new(bytes)).unwrap();
        assert_eq!(list_compound, recreated_struct);
    }

    #[test]
    fn nbt_arrays() {
        #[derive(Serialize)]
        struct Tagged {
            #[serde(serialize_with = "nbt_long_array")]
            l: [i64; 1],
            #[serde(serialize_with = "nbt_int_array")]
            i: [i32; 1],
            #[serde(serialize_with = "nbt_byte_array")]
            b: [u8; 1],
        }
        #[derive(Serialize)]
        struct NotTagged {
            l: [i64; 1],
            i: [i32; 1],
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
    fn tuple_fail() {
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
        }
    }

    #[test]
    fn tuple_ok() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct GoodData {
            x: (i32, i32),
        }

        let value = GoodData { x: (1, 2) };
        let mut bytes = Vec::new();
        to_bytes(&value, &mut bytes).unwrap();

        let reconstructed = from_bytes(Cursor::new(bytes)).unwrap();
        assert_eq!(value, reconstructed);
    }

    // --- Edge case and robustness tests ---

    #[test]
    fn empty_string_field() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct EmptyStr {
            value: String,
        }
        let test = EmptyStr {
            value: String::new(),
        };
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let result: EmptyStr = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert_eq!(test, result);
    }

    #[test]
    fn long_string_value() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct LongStr {
            value: String,
        }
        // NBT strings are prefixed with a u16 length, max 65535 bytes
        let long_value = "A".repeat(10_000);
        let test = LongStr {
            value: long_value.clone(),
        };
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let result: LongStr = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert_eq!(result.value, long_value);
    }

    #[test]
    fn deeply_nested_compound() {
        use crate::compound::NbtCompound;
        use crate::tag::NbtTag;

        // Build a 50-level deep nested compound
        let depth = 50;
        let mut innermost = NbtCompound::new();
        innermost.put_int("value", 42);

        let mut current = NbtTag::Compound(innermost);
        for i in 0..depth {
            let mut parent = NbtCompound::new();
            parent.put(&format!("level_{i}"), current);
            current = NbtTag::Compound(parent);
        }

        if let NbtTag::Compound(root) = &current {
            let nbt = Nbt::new(String::new(), root.clone());
            let bytes = nbt.write();

            let mut cursor = Cursor::new(bytes.to_vec());
            let mut reader = crate::deserializer::NbtReadHelper::new(&mut cursor);
            let parsed = Nbt::read(&mut reader).unwrap();

            // Navigate down to the value
            let mut tag = &NbtTag::Compound(parsed.root_tag);
            for i in (0..depth).rev() {
                if let NbtTag::Compound(c) = tag {
                    tag = c.get(&format!("level_{i}")).unwrap();
                } else {
                    panic!("Expected compound at level {i}");
                }
            }
            if let NbtTag::Compound(c) = tag {
                assert_eq!(c.get_int("value"), Some(42));
            } else {
                panic!("Expected innermost compound");
            }
        }
    }

    #[test]
    fn large_byte_array() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct BigByteArray {
            #[serde(serialize_with = "nbt_byte_array")]
            data: Vec<u8>,
        }
        let test = BigByteArray {
            data: vec![0xAB; 65536],
        };
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let result: BigByteArray = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert_eq!(test, result);
    }

    #[test]
    fn large_int_array() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct BigIntArray {
            #[serde(serialize_with = "nbt_int_array")]
            data: Vec<i32>,
        }
        let test = BigIntArray {
            data: (0..10_000).collect(),
        };
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let result: BigIntArray = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert_eq!(test, result);
    }

    #[test]
    fn large_long_array() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct BigLongArray {
            #[serde(serialize_with = "nbt_long_array")]
            data: Vec<i64>,
        }
        let test = BigLongArray {
            data: (0..10_000).map(|i| i * 1_000_000).collect(),
        };
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let result: BigLongArray = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert_eq!(test, result);
    }

    #[test]
    fn boundary_numeric_values() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Boundaries {
            byte_min: i8,
            byte_max: i8,
            short_min: i16,
            short_max: i16,
            int_min: i32,
            int_max: i32,
            long_min: i64,
            long_max: i64,
            float_pos_inf: f32,
            float_neg_inf: f32,
            double_pos_inf: f64,
            double_neg_inf: f64,
        }
        let test = Boundaries {
            byte_min: i8::MIN,
            byte_max: i8::MAX,
            short_min: i16::MIN,
            short_max: i16::MAX,
            int_min: i32::MIN,
            int_max: i32::MAX,
            long_min: i64::MIN,
            long_max: i64::MAX,
            float_pos_inf: f32::INFINITY,
            float_neg_inf: f32::NEG_INFINITY,
            double_pos_inf: f64::INFINITY,
            double_neg_inf: f64::NEG_INFINITY,
        };
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let result: Boundaries = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert_eq!(test, result);
    }

    #[test]
    fn float_nan_roundtrip() {
        #[derive(Serialize, Deserialize, Debug)]
        struct NanTest {
            f: f32,
            d: f64,
        }
        let test = NanTest {
            f: f32::NAN,
            d: f64::NAN,
        };
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let result: NanTest = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert!(result.f.is_nan());
        assert!(result.d.is_nan());
    }

    #[test]
    fn empty_compound_roundtrip() {
        use crate::compound::NbtCompound;

        let empty = NbtCompound::new();
        let nbt = Nbt::new(String::new(), empty);
        let bytes = nbt.write();

        let mut cursor = Cursor::new(bytes.to_vec());
        let mut reader = crate::deserializer::NbtReadHelper::new(&mut cursor);
        let parsed = Nbt::read(&mut reader).unwrap();
        assert!(parsed.root_tag.is_empty());
    }

    #[test]
    fn many_fields_compound() {
        use crate::compound::NbtCompound;

        let mut compound = NbtCompound::new();
        for i in 0..500 {
            compound.put_int(&format!("field_{i}"), i);
        }
        let nbt = Nbt::new(String::new(), compound);
        let bytes = nbt.write();

        let mut cursor = Cursor::new(bytes.to_vec());
        let mut reader = crate::deserializer::NbtReadHelper::new(&mut cursor);
        let parsed = Nbt::read(&mut reader).unwrap();

        for i in 0..500 {
            assert_eq!(parsed.root_tag.get_int(&format!("field_{i}")), Some(i));
        }
    }

    #[test]
    fn truncated_nbt_data() {
        // A valid NBT header but truncated before the compound ends
        let partial_bytes: Vec<u8> = vec![
            0x0A, // Compound tag
            0x00, 0x00, // Empty name
            0x03, // Int tag
            0x00, 0x01, // Name length = 1
            0x78, // Name = "x"
            0x00, 0x00, // Truncated: only 2 of 4 int bytes
        ];

        let result: Result<Test, _> = from_bytes(Cursor::new(partial_bytes));
        assert!(result.is_err());
    }

    #[test]
    fn unknown_tag_id() {
        // A compound containing a tag with invalid type ID 0xFF
        let bad_bytes: Vec<u8> = vec![
            0x0A, // Compound tag
            0x00, 0x00, // Empty name
            0xFF, // Invalid tag type
            0x00, 0x01, // Name length = 1
            0x78, // Name = "x"
        ];

        let result: Result<Test, _> = from_bytes(Cursor::new(bad_bytes));
        assert!(result.is_err());
    }

    #[test]
    fn empty_bytes_error() {
        let empty: Vec<u8> = vec![];
        let result: Result<Test, _> = from_bytes(Cursor::new(empty));
        assert!(result.is_err());
    }

    #[test]
    fn special_characters_in_strings() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct SpecialChars {
            value: String,
        }
        let test = SpecialChars {
            value: "hello\nworld\ttab\\backslash\"quote".to_string(),
        };
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let result: SpecialChars = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert_eq!(test, result);
    }

    #[test]
    fn unicode_string() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct UnicodeStr {
            value: String,
        }
        let test = UnicodeStr {
            value: "日本語テスト 🎮 Ünïcödë".to_string(),
        };
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let result: UnicodeStr = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert_eq!(test, result);
    }

    #[test]
    fn empty_list_field() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct EmptyList {
            items: Vec<i32>,
        }
        let test = EmptyList { items: vec![] };
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let result: EmptyList = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert_eq!(test, result);
    }

    #[test]
    fn large_list_of_compounds() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Item {
            id: i32,
            name: String,
        }
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Inventory {
            items: Vec<Item>,
        }

        let items: Vec<Item> = (0..1000)
            .map(|i| Item {
                id: i,
                name: format!("item_{i}"),
            })
            .collect();
        let test = Inventory { items };

        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let result: Inventory = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
        assert_eq!(test, result);
    }
}

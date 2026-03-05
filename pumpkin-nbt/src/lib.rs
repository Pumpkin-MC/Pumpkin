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

pub mod compound;
pub mod deserializer;
pub mod nbt_compress;
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

/// Normalizes NBT data by sorting compound tag keys in lexicographical order.
///
/// This function takes raw NBT bytes, deserializes them, sorts all compound tag
/// key-value pairs by key name in lexicographical order (recursively for nested
/// structures), and then re-serializes the data back to bytes.
///
/// # Arguments
/// * `bytes` - The input NBT data as bytes
///
/// # Returns
/// * `Result<Bytes, Error>` - The normalized NBT data, or an error if deserialization/serialization fails
///
/// # Example
/// ```rust
/// use pumpkin_nbt::normalize_nbt_bytes;
/// # let nbt_data: &[u8] = &[0x0A, 0x00, 0x00, 0x00]; // Example NBT bytes
/// let normalized = normalize_nbt_bytes(&nbt_data).unwrap();
/// ```
pub fn normalize_nbt_bytes(bytes: &[u8]) -> Result<Bytes, Error> {
    use std::io::Cursor;

    // Try to deserialize as named NBT first
    let cursor = Cursor::new(bytes);
    let nbt_result = Nbt::read(&mut NbtReadHelper::new(cursor));

    if let Ok(nbt) = nbt_result {
        // Successfully parsed as named NBT
        let normalized_nbt = Nbt::new(nbt.name, nbt.root_tag.normalize());
        Ok(normalized_nbt.write())
    } else {
        // Try as unnamed NBT
        let cursor = Cursor::new(bytes);
        let nbt = Nbt::read_unnamed(&mut NbtReadHelper::new(cursor))?;
        let normalized_nbt = Nbt::new(nbt.name, nbt.root_tag.normalize());
        Ok(normalized_nbt.write_unnamed())
    }
}

#[cfg(test)]
mod test {

    use std::io::Cursor;

    use crate::Error;
    use crate::compound::NbtCompound;
    use crate::deserializer::from_bytes;
    use crate::nbt_byte_array;
    use crate::nbt_int_array;
    use crate::nbt_long_array;
    use crate::serializer::to_bytes_named;
    use crate::serializer::{WriteAdaptor, to_bytes};
    use crate::tag::NbtTag;
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
    fn wrapper_compound_lists() {
        let mut vec: Vec<NbtTag> = Vec::new();

        // These tags will be wrapped during serialization.
        vec.push(NbtTag::Int(-1823));
        vec.push(NbtTag::Int(123));
        vec.push(NbtTag::String("Not an int".to_string()));
        vec.push(NbtTag::Byte(2));

        // This compound will not, since the list is already a list of compound tags.
        // This compound cannot be unwrapped in any way, so it is preserved
        // on deserialization.
        vec.push(NbtTag::Compound({
            let mut compound = NbtCompound::new();
            compound.put_short("example", 1234);
            compound
        }));

        // This wrapper compound will be wrapped because we want to preserve the
        // original data during deserialization.
        //
        // Suppose we had {"": `tag`}. If we didn't wrap this, on deserialization,
        // we would get `tag`, which doesn't match the serialized compound tag.
        // Therefore, we must wrap it and serialize {"": {"": `tag`}}.
        // Then on deserialization, we get {"": `tag`}, which matches what we wanted
        // to serialize in the first place.
        //
        // This compound represents {"": 1L}.
        vec.push(NbtTag::Compound({
            let mut compound = NbtCompound::new();
            compound.put_long("", 1);
            compound
        }));

        let expected_bytes = [
            0x09, // List type
            0x0A, // This list is a compound tag list
            0x00, 0x00, 0x00, 0x06, // This list has 6 elements.
            // Now for parsing each compound tag:
            0x03, // Int type
            0x00, 0x00, // Empty key
            0xFF, 0xFF, 0xF8, 0xE1, // -1823
            0x00, // End
            0x03, // Int type
            0x00, 0x00, // Empty key
            0x00, 0x00, 0x00, 0x7B, // 123
            0x00, // End
            0x08, // String type
            0x00, 0x00, // Empty key
            0x00, 0x0A, // The string is 10 characters long.
            0x4E, 0x6F, 0x74, 0x20, 0x61, 0x6E, 0x20, 0x69, 0x6E, 0x74, // "Not an int"
            0x00, // End
            0x01, // Byte type
            0x00, 0x00, // Empty key
            0x02, // 2b
            0x00, // End
            // For the first (unwrapped) compound:
            0x02, // Short type
            0x00, 0x07, // The key is 7 characters long.
            0x65, 0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65, // "example"
            0x04, 0xD2, // 1234
            0x00, // End
            // For the second (wrapped) wrapper compound:
            0x0A, // Compound type
            0x00, 0x00, // Empty key
            0x04, // Long type
            0x00, 0x00, // Empty key
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // 1L
            0x00, // End
            0x00, // End
        ];

        let mut bytes = Vec::new();
        let mut write_adaptor = WriteAdaptor::new(&mut bytes);
        NbtTag::List(vec)
            .serialize(&mut write_adaptor)
            .expect("Expected serialization to succeed");

        assert_eq!(bytes, expected_bytes);
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

    #[test]
    fn test_normalize_nbt_bytes() {
        use crate::normalize_nbt_bytes;
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestStruct {
            z_field: String,
            a_field: i32,
            m_field: bool,
        }

        let test_data = TestStruct {
            z_field: "last".to_string(),
            a_field: 42,
            m_field: true,
        };

        let mut bytes = Vec::new();
        to_bytes_unnamed(&test_data, &mut bytes).unwrap();
        let normalized_bytes = normalize_nbt_bytes(&bytes).unwrap();
        let reconstructed: TestStruct =
            from_bytes_unnamed(std::io::Cursor::new(normalized_bytes.clone())).unwrap();
        assert_eq!(test_data, reconstructed);

        let normalized_again = normalize_nbt_bytes(&normalized_bytes).unwrap();
        assert_eq!(
            normalized_bytes, normalized_again,
            "Normalize should be idempotent"
        );

        let mut bytes2 = Vec::new();
        to_bytes_unnamed(&test_data, &mut bytes2).unwrap();
        let normalized_bytes2 = normalize_nbt_bytes(&bytes2).unwrap();
        assert_eq!(
            normalized_bytes, normalized_bytes2,
            "Same data should normalize to same bytes"
        );
    }

    #[test]
    fn test_normalize_nested_compounds() {
        use crate::normalize_nbt_bytes;
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Inner {
            z_inner: i32,
            a_inner: String,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Outer {
            z_outer: Inner,
            a_outer: Inner,
        }

        let test_data = Outer {
            z_outer: Inner {
                z_inner: 1,
                a_inner: "first".to_string(),
            },
            a_outer: Inner {
                z_inner: 2,
                a_inner: "second".to_string(),
            },
        };

        // Serialize to bytes
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test_data, &mut bytes).unwrap();

        // Normalize the bytes
        let normalized_bytes = normalize_nbt_bytes(&bytes).unwrap();

        // Deserialize back and verify it's the same data
        let reconstructed: Outer =
            from_bytes_unnamed(std::io::Cursor::new(normalized_bytes)).unwrap();
        assert_eq!(test_data, reconstructed);
    }

    #[test]
    fn test_normalize_with_lists() {
        use crate::normalize_nbt_bytes;
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestStruct {
            z_field: String,
            a_field: i32,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestWithList {
            z_list: Vec<TestStruct>,
            a_single: TestStruct,
        }

        let test_data = TestWithList {
            z_list: vec![
                TestStruct {
                    z_field: "item1".to_string(),
                    a_field: 1,
                },
                TestStruct {
                    z_field: "item2".to_string(),
                    a_field: 2,
                },
            ],
            a_single: TestStruct {
                z_field: "single".to_string(),
                a_field: 3,
            },
        };

        // Serialize to bytes
        let mut bytes = Vec::new();
        to_bytes_unnamed(&test_data, &mut bytes).unwrap();

        // Normalize the bytes
        let normalized_bytes = normalize_nbt_bytes(&bytes).unwrap();

        // Deserialize back and verify it's the same data
        let reconstructed: TestWithList =
            from_bytes_unnamed(std::io::Cursor::new(normalized_bytes)).unwrap();
        assert_eq!(test_data, reconstructed);
    }

    // TODO: More robust tests
}

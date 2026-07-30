//! Reading, writing, and manipulating Minecraft's Named Binary Tag (NBT) data.
//!
//! The crate supports the standard Java Edition representation, unnamed network
//! NBT, Bedrock network NBT, and gzip-compressed NBT. Data can be handled either
//! as [`Nbt`] and [`NbtCompound`] values or through Serde with [`to_bytes`] and
//! [`from_slice`].
//!
//! # Serde round trip
//!
//! ```
//! use pumpkin_nbt::{from_slice, to_bytes};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct Player {
//!     name: String,
//!     health: i16,
//! }
//!
//! let player = Player {
//!     name: "Steve".to_owned(),
//!     health: 20,
//! };
//! let mut encoded = Vec::new();
//! to_bytes(&player, &mut encoded)?;
//!
//! let decoded: Player = from_slice(&encoded)?;
//! assert_eq!(decoded, player);
//! # Ok::<(), pumpkin_nbt::Error>(())
//! ```

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

/// Compound-tag storage and construction helpers.
pub mod compound;
/// Serde and low-level NBT deserialization support.
pub mod deserializer;
/// Reading and writing gzip-compressed NBT.
pub mod nbt_compress;
/// Integration with Pumpkin's dynamic codec operations.
pub mod nbt_ops;
/// Serde and low-level NBT serialization support.
pub mod serializer;
/// The individual NBT tag types.
pub mod tag;

pub use compound::NbtCompound;
pub use deserializer::{
    from_bytes, from_bytes_bedrock, from_bytes_unnamed, from_slice, from_slice_bedrock,
    from_slice_unnamed,
};
pub use serializer::{to_bytes, to_bytes_named, to_bytes_unnamed};

// This NBT crate is inspired from CrabNBT

/// Numeric identifier for an end tag.
pub const END_ID: u8 = 0x00;
/// Numeric identifier for a byte tag.
pub const BYTE_ID: u8 = 0x01;
/// Numeric identifier for a short tag.
pub const SHORT_ID: u8 = 0x02;
/// Numeric identifier for an integer tag.
pub const INT_ID: u8 = 0x03;
/// Numeric identifier for a long tag.
pub const LONG_ID: u8 = 0x04;
/// Numeric identifier for a float tag.
pub const FLOAT_ID: u8 = 0x05;
/// Numeric identifier for a double tag.
pub const DOUBLE_ID: u8 = 0x06;
/// Numeric identifier for a byte-array tag.
pub const BYTE_ARRAY_ID: u8 = 0x07;
/// Numeric identifier for a string tag.
pub const STRING_ID: u8 = 0x08;
/// Numeric identifier for a list tag.
pub const LIST_ID: u8 = 0x09;
/// Numeric identifier for a compound tag.
pub const COMPOUND_ID: u8 = 0x0A;
/// Numeric identifier for an integer-array tag.
pub const INT_ARRAY_ID: u8 = 0x0B;
/// Numeric identifier for a long-array tag.
pub const LONG_ARRAY_ID: u8 = 0x0C;

/// Maximum number of elements accepted when decoding a list or array.
pub const MAX_ARRAY_LENGTH: usize = 2_000_000;

/// Errors produced while reading, writing, or converting NBT data.
#[derive(Error, Debug)]
pub enum Error {
    /// The root tag was not a compound tag and contains the reported tag ID.
    #[error("The root tag of the NBT file is not a compound tag. Received tag id: {0}")]
    NoRootCompound(u8),
    /// A tag ID not defined by the NBT format was encountered.
    #[error("Encountered an unknown NBT tag id: {0}.")]
    UnknownTagId(u8),
    /// A Java CESU-8 string could not be decoded.
    #[error("Failed to Cesu 8 Decode")]
    Cesu8DecodingError,
    /// A string could not be decoded as UTF-8.
    #[error("Failed to UTF-8 Decode")]
    Utf8DecodingError,
    /// Serde reported an invalid value or serializer state.
    #[error("Serde error: {0}")]
    SerdeError(String),
    /// The requested Rust type has no NBT representation.
    #[error("NBT doesn't support this type: {0}")]
    UnsupportedType(String),
    /// The underlying reader or writer returned an I/O error.
    #[error("NBT reading was cut short: {0}")]
    Incomplete(io::Error),
    /// A list or array declared a negative element count.
    #[error("Negative list length: {0}")]
    NegativeLength(i32),
    /// A string, list, or array exceeded the supported length.
    #[error("Length too large: {0}")]
    LargeLength(usize),
    /// A Bedrock variable-length integer exceeded its maximum encoded size.
    #[error("Failed to decode varint - value too large")]
    VarIntTooLarge,
    /// A Bedrock variable-length long exceeded its maximum encoded size.
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

/// A complete NBT document containing a named root compound.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Nbt {
    /// Name stored alongside the root compound.
    pub name: String,
    /// Root compound containing the document's tags.
    pub root_tag: NbtCompound,
}

impl Nbt {
    /// Creates a document from a root name and compound.
    #[must_use]
    pub const fn new(name: String, tag: NbtCompound) -> Self {
        Self {
            name,
            root_tag: tag,
        }
    }

    /// Reads a named NBT document from a format-specific reader.
    ///
    /// Returns [`Error::NoRootCompound`] when the first tag is not a compound.
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

    /// Reads an NBT document that omits the root compound's name.
    ///
    /// The returned document has an empty [`Self::name`].
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

    /// Serializes this document using the Java Edition NBT representation.
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

    /// Serializes this document using the Bedrock network NBT representation.
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

    /// Writes this document in the Java Edition representation.
    pub fn write_to_writer<W: Write>(self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.write())?;
        Ok(())
    }

    /// Writes this document in the Bedrock network representation.
    pub fn write_to_writer_bedrock<W: Write>(self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.write_bedrock())?;
        Ok(())
    }

    /// Serializes this document without the root compound's name.
    #[must_use]
    pub fn write_unnamed(self) -> Bytes {
        let mut bytes = Vec::new();
        let mut writer = NbtWriteHelperJava::new(&mut bytes);

        writer.write_u8(COMPOUND_ID).unwrap();
        self.root_tag.serialize_content(&mut writer).unwrap();

        bytes.into()
    }

    /// Writes this document without the root compound's name.
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
        #[doc = "Serializes a sequence using its specialized NBT array representation."]
        #[doc = ""]
        #[doc = "Use this function with Serde's `serialize_with` field attribute."]
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
    use crate::compound::NbtCompound;
    use crate::deserializer::from_bytes;
    use crate::deserializer::from_bytes_bedrock;
    use crate::nbt_byte_array;
    use crate::nbt_int_array;
    use crate::nbt_long_array;
    use crate::serializer::NbtWriteHelperBedrock;
    use crate::serializer::to_bytes_bedrock;
    use crate::serializer::to_bytes_named;
    use crate::serializer::to_bytes_named_bedrock;
    use crate::serializer::{NbtWriteHelperJava, to_bytes};
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

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct BorrowedTest<'a> {
        byte: i8,
        string: &'a str,
    }

    #[test]
    fn zero_alloc_from_slice() {
        let test = Test {
            byte: 123,
            short: 1342,
            int: 4313,
            long: 34,
            float: 1.00,
            string: "Zero alloc NBT".to_string(),
        };

        let mut bytes = Vec::new();
        to_bytes_unnamed(&test, &mut bytes).unwrap();
        let borrowed: BorrowedTest = crate::from_slice_unnamed(&bytes).unwrap();

        assert_eq!(borrowed.byte, 123);
        assert_eq!(borrowed.string, "Zero alloc NBT");
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

    #[test]
    fn simple_ser_de_unnamed_bedrock() {
        let test = Test {
            byte: 123,
            short: 1342,
            int: 4313,
            long: 34,
            float: 1.00,
            string: "Hello test".to_string(),
        };

        // Bedrock doesn't actually use unnamed NBT (AFAIK). `to_bytes_bedrock` actually encodes empty name.
        let mut bytes = Vec::new();
        to_bytes_bedrock(&test, &mut bytes).unwrap();
        let recreated_struct: Test = from_bytes_bedrock(Cursor::new(bytes)).unwrap();

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
    fn simple_ser_de_array_bedrock() {
        let test = TestArray {
            byte_array: vec![0, 3, 2],
            int_array: vec![13, 1321, 2],
            long_array: vec![1, 0, 200301, 1],
        };

        let mut bytes = Vec::new();
        to_bytes_bedrock(&test, &mut bytes).unwrap();
        let recreated_struct: TestArray = from_bytes_bedrock(Cursor::new(bytes)).unwrap();

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
    fn simple_ser_de_named_bedrock() {
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
        to_bytes_named_bedrock(&test, name, &mut bytes).unwrap();
        let recreated_struct: Test = from_bytes_bedrock(Cursor::new(bytes)).unwrap();

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

    #[test]
    fn simple_ser_de_array_named_bedrock() {
        let name = String::from("Test");
        let test = TestArray {
            byte_array: vec![0, 3, 2],
            int_array: vec![13, 1321, 2],
            long_array: vec![1, 0, 200301, 1],
        };

        let mut bytes = Vec::new();
        to_bytes_named_bedrock(&test, name, &mut bytes).unwrap();
        let recreated_struct: TestArray = from_bytes_bedrock(Cursor::new(bytes)).unwrap();

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
    fn list_bedrock() {
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
        to_bytes_bedrock(&list_compound, &mut bytes).unwrap();
        let recreated_struct: TestList = from_bytes_bedrock(Cursor::new(bytes)).unwrap();
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
    fn list_named_bedrock() {
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
        to_bytes_named_bedrock(&list_compound, "a".to_string(), &mut bytes).unwrap();
        let recreated_struct: TestList = from_bytes_bedrock(Cursor::new(bytes)).unwrap();
        assert_eq!(list_compound, recreated_struct);
    }

    #[test]
    fn wrapper_compound_lists() {
        let mut vec: Vec<NbtTag> = Vec::new();

        // These tags will be wrapped during serialization.
        vec.push(NbtTag::Int(-1823));
        vec.push(NbtTag::Int(123));
        vec.push(NbtTag::String("Not an int".into()));
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
        let mut write_adaptor = NbtWriteHelperJava::new(&mut bytes);
        NbtTag::List(vec)
            .serialize(&mut write_adaptor)
            .expect("Expected serialization to succeed");

        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn wrapper_compound_lists_bedrock() {
        let mut vec: Vec<NbtTag> = Vec::new();

        // These tags will be wrapped during serialization.
        vec.push(NbtTag::Int(-1823));
        vec.push(NbtTag::Int(123));
        vec.push(NbtTag::String("Not an int".into()));
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
            0xC,  // This list has 6 elements.
            // Now for parsing each compound tag:
            0x03, // Int type
            0x00, // Empty key
            0xBD, 0x1C, // -1823
            0x00, // End
            0x03, // Int type
            0x00, // Empty key
            0xF6, 0x01, // 123
            0x00, // End
            0x08, // String type
            0x00, // Empty key
            0x0A, // The string is 10 characters long.
            0x4E, 0x6F, 0x74, 0x20, 0x61, 0x6E, 0x20, 0x69, 0x6E, 0x74, // "Not an int"
            0x00, // End
            0x01, // Byte type
            0x00, // Empty key
            0x02, // 2b
            0x00, // End
            // For the first (unwrapped) compound:
            0x02, // Short type
            0x07, // The key is 7 characters long.
            0x65, 0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65, // "example"
            0xD2, 0x04, // 1234
            0x00, // End
            // For the second (wrapped) wrapper compound:
            0x0A, // Compound type
            0x00, // Empty key
            0x04, // Long type
            0x00, // Empty key
            0x02, // 1L
            0x00, // End
            0x00, // End
        ];

        let mut bytes = Vec::new();
        let mut write_adaptor = NbtWriteHelperBedrock::new(&mut bytes);
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
    fn nbt_arrays_bedrock() {
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
            0x00, // Empty root name
            0x0C, // Long Array Type
            0x01, // Key length
            0x6C, // Key (l)
            0x02, // Array Length
            0x00, // Value(s)
            0x0B, // Int Array Tag
            0x01, // Key length
            0x69, // Key (i)
            0x02, // Array Length
            0x00, // Value(s)
            0x07, // Byte Array Tag
            0x01, // Key length
            0x62, // Key (b)
            0x02, // Array Length
            0x00, // Value(s)
            0x00, // End Tag
        ];

        let mut bytes = Vec::new();
        to_bytes_bedrock(&value, &mut bytes).unwrap();
        assert_eq!(bytes, expected_bytes);

        let value = NotTagged {
            l: [0],
            i: [0],
            b: [0],
        };
        let expected_bytes = [
            0x0A, // Component Tag
            0x00, // Empty root name
            0x09, // List Tag
            0x01, // Key length
            0x6C, // Key (l)
            0x04, // Array Type
            0x02, // Array Length
            0x00, // Value(s)
            0x09, // List Tag
            0x01, // Key length
            0x69, // Key (i)
            0x03, // Array Type
            0x02, // Array Length
            0x00, // Value(s)
            0x09, // List Tag
            0x01, // Key length
            0x62, // Key (b)
            0x01, // Array Type
            0x02, // Array Length
            0x00, // Value(s)
            0x00, // End Tag
        ];

        let mut bytes = Vec::new();
        to_bytes_bedrock(&value, &mut bytes).unwrap();
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
    fn tuple_fail_bedrock() {
        #[derive(Serialize)]
        struct BadData {
            x: (i32, i64),
        }

        let value = BadData { x: (0, 0) };
        let mut bytes = Vec::new();
        let err = to_bytes_bedrock(&value, &mut bytes);

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
    fn tuple_ok_bedrock() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct GoodData {
            x: (i32, i32),
        }

        let value = GoodData { x: (1, 2) };
        let mut bytes = Vec::new();
        to_bytes_bedrock(&value, &mut bytes).unwrap();

        let reconstructed = from_bytes_bedrock(Cursor::new(bytes)).unwrap();
        assert_eq!(value, reconstructed);
    }

    // TODO: More robust tests
}

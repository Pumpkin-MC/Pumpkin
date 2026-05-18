use serde::ser::Impossible;
use serde::{Serialize, ser};
use std::io::Write;

use crate::{
    BYTE_ARRAY_ID, BYTE_ID, COMPOUND_ID, DOUBLE_ID, END_ID, Error, FLOAT_ID, INT_ARRAY_ID, INT_ID,
    LIST_ID, LONG_ARRAY_ID, LONG_ID, NBT_ARRAY_TAG, NBT_BYTE_ARRAY_TAG, NBT_INT_ARRAY_TAG,
    NBT_LONG_ARRAY_TAG, SHORT_ID, STRING_ID,
};

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! define_write_number_be {
    ($name:ident, $type:ty) => {
        fn $name(&mut self, value: $type) -> Result<()> {
            let buf = value.to_be_bytes();
            self.writer.write_all(&buf).map_err(Error::Incomplete)?;
            Ok(())
        }
    };
}

macro_rules! define_write_number_le {
    ($name:ident, $type:ty) => {
        fn $name(&mut self, value: $type) -> Result<()> {
            let buf = value.to_le_bytes();
            self.writer.write_all(&buf).map_err(Error::Incomplete)?;
            Ok(())
        }
    };
}

pub trait NbtWriteHelper {
    type Writer: Write;

    fn writer(&mut self) -> &mut Self::Writer;
    fn write_u8(&mut self, value: u8) -> Result<()>;
    fn write_i8(&mut self, value: i8) -> Result<()>;
    fn write_i16(&mut self, value: i16) -> Result<()>;
    fn write_i32(&mut self, value: i32) -> Result<()>;
    fn write_i64(&mut self, value: i64) -> Result<()>;
    fn write_f32(&mut self, value: f32) -> Result<()>;
    fn write_f64(&mut self, value: f64) -> Result<()>;
    fn write_string(&mut self, value: &str) -> Result<()>;

    fn write_slice(&mut self, value: &[u8]) -> Result<()> {
        self.writer().write_all(value).map_err(Error::Incomplete)?;
        Ok(())
    }
}

pub struct NbtWriteHelperJava<W: Write> {
    writer: W,
}

impl<W: Write> NbtWriteHelperJava<W> {
    pub const fn new(w: W) -> Self {
        Self { writer: w }
    }
}

impl<W: Write> NbtWriteHelperJava<W> {
    define_write_number_be!(write_string_len, u16);
}

impl<W: Write> NbtWriteHelper for NbtWriteHelperJava<W> {
    type Writer = W;

    fn writer(&mut self) -> &mut Self::Writer {
        &mut self.writer
    }

    define_write_number_be!(write_u8, u8);
    define_write_number_be!(write_i8, i8);
    define_write_number_be!(write_i16, i16);
    define_write_number_be!(write_i32, i32);
    define_write_number_be!(write_i64, i64);
    define_write_number_be!(write_f32, f32);
    define_write_number_be!(write_f64, f64);

    fn write_string(&mut self, value: &str) -> Result<()> {
        let java_string = cesu8::to_java_cesu8(value);
        let len = java_string.len();
        if len > u16::MAX as usize {
            return Err(Error::LargeLength(len));
        }

        self.write_string_len(len as u16)?;
        self.writer
            .write_all(&java_string)
            .map_err(Error::Incomplete)?;
        Ok(())
    }
}

pub struct NbtWriteHelperBedrock<W: Write> {
    writer: W,
}

impl<W: Write> NbtWriteHelperBedrock<W> {
    pub const fn new(w: W) -> Self {
        Self { writer: w }
    }
}

impl<W: Write> NbtWriteHelperBedrock<W> {
    fn write_var_u32(&mut self, mut value: u32) -> Result<()> {
        // LEB128
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.write_u8(byte)?;
            if value == 0 {
                break;
            }
        }
        Ok(())
    }

    fn write_var_i32(&mut self, value: i32) -> Result<()> {
        // ZigZag
        self.write_var_u32(((value << 1) ^ (value >> 31)) as u32)
    }

    fn write_var_u64(&mut self, mut value: u64) -> Result<()> {
        // LEB128
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.write_u8(byte)?;
            if value == 0 {
                break;
            }
        }
        Ok(())
    }

    fn write_var_i64(&mut self, value: i64) -> Result<()> {
        // ZigZag
        self.write_var_u64(((value << 1) ^ (value >> 63)) as u64)
    }

    fn write_string_len(&mut self, value: u32) -> Result<()> {
        self.write_var_u32(value)
    }
}

impl<W: Write> NbtWriteHelper for NbtWriteHelperBedrock<W> {
    type Writer = W;

    fn writer(&mut self) -> &mut Self::Writer {
        &mut self.writer
    }

    define_write_number_le!(write_u8, u8);
    define_write_number_le!(write_i8, i8);
    define_write_number_le!(write_i16, i16);
    fn write_i32(&mut self, value: i32) -> Result<()> {
        self.write_var_i32(value)
    }
    fn write_i64(&mut self, value: i64) -> Result<()> {
        self.write_var_i64(value)
    }
    define_write_number_le!(write_f32, f32);
    define_write_number_le!(write_f64, f64);

    fn write_string(&mut self, value: &str) -> Result<()> {
        let bedrock_string = value.as_bytes();
        let len = bedrock_string.len();
        if len > u32::MAX as usize {
            return Err(Error::LargeLength(len));
        }

        self.write_string_len(len as u32)?;
        self.writer
            .write_all(bedrock_string)
            .map_err(Error::Incomplete)?;
        Ok(())
    }
}

pub struct Serializer<W: NbtWriteHelper> {
    output: W,
    state: State,
    handled_root: bool,
    expected_list_tag: u8,
}

impl<W: NbtWriteHelper> Serializer<W> {
    pub const fn new(output: W, name: Option<String>) -> Self {
        Self {
            output,
            state: State::Root(name),
            handled_root: false,
            expected_list_tag: 0,
        }
    }
}

// NBT has a different order of things than most other formats,
// so I use `State` to keep what the serializer has to do, and some information like the field name.
#[derive(Clone, Debug, PartialEq)]
enum State {
    // In network NBT, the root name is not present.
    Root(Option<String>),
    Named(String),
    // Used by maps to check if key is a `String`.
    MapKey,
    FirstListElement {
        len: i32,
    },
    ListElement,
    CheckedListElement,
    Array {
        name: String,
        array_type: &'static str,
    },
}

impl<W: NbtWriteHelper> Serializer<W> {
    fn parse_state(&mut self, tag: u8) -> Result<()> {
        match &mut self.state {
            State::Named(name) | State::Array { name, .. } => {
                self.output.write_u8(tag)?;
                self.output.write_string(name)?;
            }
            State::FirstListElement { len } => {
                self.output.write_u8(tag)?;
                self.output.write_i32(*len)?;
                self.expected_list_tag = tag;
            }
            State::MapKey => {
                if tag != STRING_ID {
                    return Err(Error::SerdeError(format!(
                        "Map key can only be `String`, not {tag}"
                    )));
                }
            }
            State::ListElement => {
                // Rust rules mandate this is all the same type
            }
            State::CheckedListElement => {
                if tag != self.expected_list_tag {
                    return Err(Error::SerdeError(format!(
                        "List values must all be of the same type! Expected {} but found {}!",
                        self.expected_list_tag, tag
                    )));
                }
            }
            State::Root(root_name) => {
                if self.handled_root {
                    return Err(Error::SerdeError(
                        "Invalid state: already handled root component!".to_string(),
                    ));
                }
                if tag != COMPOUND_ID {
                    return Err(Error::SerdeError(format!(
                        "Invalid state: root is not a `Compound`! ({tag})"
                    )));
                }
                self.handled_root = true;

                self.output.write_u8(tag)?;
                if let Some(root_name) = root_name {
                    self.output.write_string(root_name)?;
                }
            }
        }
        Ok(())
    }
}

/// Serializes struct using Serde Serializer to unnamed (network) NBT
pub fn to_bytes_unnamed<T: Serialize>(value: &T, w: impl Write) -> Result<()> {
    let mut serializer = Serializer::new(NbtWriteHelperJava::new(w), None);
    value.serialize(&mut serializer)?;
    Ok(())
}

/// Serializes struct using Serde Serializer to normal NBT
pub fn to_bytes_named<T: Serialize>(value: &T, name: String, w: impl Write) -> Result<()> {
    let mut serializer = Serializer::new(NbtWriteHelperJava::new(w), Some(name));
    value.serialize(&mut serializer)?;
    Ok(())
}

/// Serializes struct using Serde Serializer to Bedrock network NBT
pub fn to_bytes_named_bedrock<T: Serialize>(value: &T, name: String, w: impl Write) -> Result<()> {
    let mut serializer = Serializer::new(NbtWriteHelperBedrock::new(w), Some(name));
    value.serialize(&mut serializer)?;
    Ok(())
}

pub fn to_bytes<T: Serialize>(value: &T, w: impl Write) -> Result<()> {
    to_bytes_named(value, String::new(), w)
}

pub fn to_bytes_bedrock<T: Serialize>(value: &T, w: impl Write) -> Result<()> {
    to_bytes_named_bedrock(value, String::new(), w)
}

impl<W: NbtWriteHelper> ser::Serializer for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_i8(i8::from(v))?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.parse_state(BYTE_ID)?;
        self.output.write_i8(v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.parse_state(SHORT_ID)?;
        self.output.write_i16(v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.parse_state(INT_ID)?;
        self.output.write_i32(v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.parse_state(LONG_ID)?;
        self.output.write_i64(v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        if let State::Named(_) = self.state {
            Err(Error::UnsupportedType(
                "u8; NBT only supports signed values".to_string(),
            ))
        } else {
            self.parse_state(BYTE_ID)?;
            self.output.write_u8(v)?;
            Ok(())
        }
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_i16(v as i16)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_i32(v as i32)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.parse_state(FLOAT_ID)?;
        self.output.write_f32(v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.parse_state(DOUBLE_ID)?;
        self.output.write_f64(v)?;
        Ok(())
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::UnsupportedType("char".to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.parse_state(STRING_ID)?;

        if self.state == State::MapKey {
            self.state = State::Named(v.to_string());
        } else {
            self.output.write_string(v)?;
        }

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.parse_state(LIST_ID)?;
        self.output.write_u8(BYTE_ID)?;

        let len = v.len();
        if len > i32::MAX as usize {
            return Err(Error::LargeLength(len));
        }

        self.output.write_i32(len as i32)?;
        self.output.write_slice(v)?;
        Ok(())
    }

    // Just skip serializing if value is none
    fn serialize_none(self) -> Result<()> {
        match self.state {
            State::FirstListElement { .. } | State::ListElement | State::CheckedListElement => Err(
                Error::SerdeError("NBT lists do not support null/none values".to_string()),
            ),
            _ => Ok(()),
        }
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::UnsupportedType("unit struct".to_string()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)?;
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<()> {
        Err(Error::UnsupportedType("newtype struct".to_string()))
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()> {
        if name == NBT_ARRAY_TAG {
            let name = match self.state {
                State::Named(ref name) => name.clone(),
                _ => return Err(Error::SerdeError("Invalid `Serializer` state!".to_string())),
            };

            self.state = State::Array {
                name,
                array_type: variant,
            };
        } else {
            return Err(Error::UnsupportedType("newtype variant".to_string()));
        }
        value.serialize(self)?;

        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let Some(len) = len else {
            return Err(Error::SerdeError(
                "The length of the sequence must be known first!".to_string(),
            ));
        };
        if len > i32::MAX as usize {
            return Err(Error::LargeLength(len));
        }

        if let State::Array { array_type, .. } = &mut self.state {
            let (id, expected_tag) = match *array_type {
                NBT_BYTE_ARRAY_TAG => (BYTE_ARRAY_ID, BYTE_ID),
                NBT_INT_ARRAY_TAG => (INT_ARRAY_ID, INT_ID),
                NBT_LONG_ARRAY_TAG => (LONG_ARRAY_ID, LONG_ID),
                _ => {
                    return Err(Error::SerdeError(
                        "Array supports only `byte`, `int`, and `long`".to_string(),
                    ));
                }
            };

            self.parse_state(id)?;

            self.output.write_i32(len as i32)?;

            // We can mark anything as an NBT array list, so mark as needed to be checked.
            self.expected_list_tag = expected_tag;
            self.state = State::CheckedListElement;
        } else {
            self.parse_state(LIST_ID)?;
            self.state = State::FirstListElement { len: len as i32 };
            if len == 0 {
                // If we have no elements, the `FirstListElement` state will never be invoked, so
                // write the (unknown) list type and length here.
                self.output.write_u8(END_ID)?;
                self.output.write_i32(0)?;
            }
        }

        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::UnsupportedType("tuple struct".to_string()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedType("tuple variant".to_string()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.parse_state(COMPOUND_ID)?;
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.parse_state(COMPOUND_ID)?;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedType("struct variant".to_string()))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<W: NbtWriteHelper> ser::SerializeTuple for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), Self::Error> {
        value.serialize(&mut **self)?;
        self.state = State::CheckedListElement;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W: NbtWriteHelper> ser::SerializeSeq for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)?;
        self.state = State::ListElement;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W: NbtWriteHelper> ser::SerializeStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        self.state = State::Named(key.to_string());
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write_u8(END_ID)?;
        Ok(())
    }
}

impl<W: NbtWriteHelper> ser::SerializeMap for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(
        &mut self,
        key: &T,
    ) -> std::result::Result<(), Self::Error> {
        self.state = State::MapKey;
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), Self::Error> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write_u8(END_ID)?;
        Ok(())
    }
}

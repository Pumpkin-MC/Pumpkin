use compound::NbtCompound;
use deserializer::NbtReadHelper;
use io::Read;
use serde::{Deserialize, Serialize};
use serializer::WriteAdaptor;

use crate::{
    BYTE_ARRAY_ID, BYTE_ID, COMPOUND_ID, DOUBLE_ID, END_ID, Error, FLOAT_ID, INT_ARRAY_ID, INT_ID,
    LIST_ID, LONG_ARRAY_ID, LONG_ID, SHORT_ID, STRING_ID, Seek, Write, compound, deserializer,
    get_nbt_string, io, nbt_byte_array, nbt_int_array, nbt_long_array, serializer,
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum NbtTag {
    End = END_ID,
    Byte(i8) = BYTE_ID,
    Short(i16) = SHORT_ID,
    Int(i32) = INT_ID,
    Long(i64) = LONG_ID,
    Float(f32) = FLOAT_ID,
    Double(f64) = DOUBLE_ID,
    ByteArray(Box<[u8]>) = BYTE_ARRAY_ID,
    String(String) = STRING_ID,
    List(Vec<Self>) = LIST_ID,
    Compound(NbtCompound) = COMPOUND_ID,
    IntArray(Vec<i32>) = INT_ARRAY_ID,
    LongArray(Vec<i64>) = LONG_ARRAY_ID,
}

impl NbtTag {
    /// Returns the numeric id associated with the data type.
    #[must_use]
    pub const fn get_type_id(&self) -> u8 {
        // Safety: Since Self is repr(u8), it is guaranteed to hold the discriminant in the first byte
        // See https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        unsafe { *std::ptr::from_ref::<Self>(self).cast::<u8>() }
    }

    pub fn serialize<W: Write>(self, w: &mut WriteAdaptor<W>) -> serializer::Result<()> {
        w.write_u8_be(self.get_type_id())?;
        self.serialize_data(w)?;
        Ok(())
    }

    pub fn write_string<W: Write>(string: &str, w: &mut WriteAdaptor<W>) -> serializer::Result<()> {
        let java_string = cesu8::to_java_cesu8(string);
        let len = java_string.len();
        if len > u16::MAX as usize {
            return Err(Error::LargeLength(len));
        }

        w.write_u16_be(len as u16)?;
        w.write_slice(&java_string)?;
        Ok(())
    }

    pub fn serialize_data<W: Write>(self, w: &mut WriteAdaptor<W>) -> serializer::Result<()> {
        match self {
            Self::End => {}
            Self::Byte(byte) => w.write_i8_be(byte)?,
            Self::Short(short) => w.write_i16_be(short)?,
            Self::Int(int) => w.write_i32_be(int)?,
            Self::Long(long) => w.write_i64_be(long)?,
            Self::Float(float) => w.write_f32_be(float)?,
            Self::Double(double) => w.write_f64_be(double)?,
            Self::ByteArray(byte_array) => {
                let len = byte_array.len();
                if len > i32::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                w.write_i32_be(len as i32)?;
                w.write_slice(&byte_array)?;
            }
            Self::String(string) => {
                Self::write_string(&string, w)?;
            }
            Self::List(list) => {
                let len = list.len();
                if len > i32::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                w.write_u8_be(list.first().unwrap_or(&Self::End).get_type_id())?;
                w.write_i32_be(len as i32)?;
                for nbt_tag in list {
                    nbt_tag.serialize_data(w)?;
                }
            }
            Self::Compound(compound) => {
                compound.serialize_content(w)?;
            }
            Self::IntArray(int_array) => {
                let len = int_array.len();
                if len > i32::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                w.write_i32_be(len as i32)?;
                for int in int_array {
                    w.write_i32_be(int)?;
                }
            }
            Self::LongArray(long_array) => {
                let len = long_array.len();
                if len > i32::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                w.write_i32_be(len as i32)?;

                for long in long_array {
                    w.write_i64_be(long)?;
                }
            }
        }
        Ok(())
    }

    pub fn deserialize<R: Read + Seek>(reader: &mut NbtReadHelper<R>) -> Result<Self, Error> {
        let tag_id = reader.get_u8_be()?;
        Self::deserialize_data(reader, tag_id)
    }

    pub fn skip_data<R: Read + Seek>(
        reader: &mut NbtReadHelper<R>,
        tag_id: u8,
    ) -> Result<(), Error> {
        match tag_id {
            END_ID => Ok(()),
            BYTE_ID => reader.skip_bytes(1),
            SHORT_ID => reader.skip_bytes(2),
            INT_ID | FLOAT_ID => reader.skip_bytes(4),
            LONG_ID | DOUBLE_ID => reader.skip_bytes(8),
            BYTE_ARRAY_ID => {
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }
                reader.skip_bytes(i64::from(len))
            }
            STRING_ID => {
                let len = reader.get_u16_be()?;
                reader.skip_bytes(i64::from(len))
            }
            LIST_ID => {
                let tag_type_id = reader.get_u8_be()?;
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                for _ in 0..len {
                    Self::skip_data(reader, tag_type_id)?;
                }

                Ok(())
            }
            COMPOUND_ID => NbtCompound::skip_content(reader),
            INT_ARRAY_ID => {
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                reader.skip_bytes(i64::from(len) * 4)
            }
            LONG_ARRAY_ID => {
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                reader.skip_bytes(i64::from(len) * 8)
            }
            _ => Err(Error::UnknownTagId(tag_id)),
        }
    }

    pub fn deserialize_data<R: Read + Seek>(
        reader: &mut NbtReadHelper<R>,
        tag_id: u8,
    ) -> Result<Self, Error> {
        match tag_id {
            END_ID => Ok(Self::End),
            BYTE_ID => {
                let byte = reader.get_i8_be()?;
                Ok(Self::Byte(byte))
            }
            SHORT_ID => {
                let short = reader.get_i16_be()?;
                Ok(Self::Short(short))
            }
            INT_ID => {
                let int = reader.get_i32_be()?;
                Ok(Self::Int(int))
            }
            LONG_ID => {
                let long = reader.get_i64_be()?;
                Ok(Self::Long(long))
            }
            FLOAT_ID => {
                let float = reader.get_f32_be()?;
                Ok(Self::Float(float))
            }
            DOUBLE_ID => {
                let double = reader.get_f64_be()?;
                Ok(Self::Double(double))
            }
            BYTE_ARRAY_ID => {
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                let byte_array = reader.read_boxed_slice(len as usize)?;
                Ok(Self::ByteArray(byte_array))
            }
            STRING_ID => Ok(Self::String(get_nbt_string(reader)?)),
            LIST_ID => {
                let tag_type_id = reader.get_u8_be()?;
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                let mut list = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let tag = Self::deserialize_data(reader, tag_type_id)?;
                    assert_eq!(tag.get_type_id(), tag_type_id);
                    list.push(tag);
                }
                Ok(Self::List(list))
            }
            COMPOUND_ID => Ok(Self::Compound(NbtCompound::deserialize_content(reader)?)),
            INT_ARRAY_ID => {
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                let len = len as usize;
                let mut int_array = Vec::with_capacity(len);
                for _ in 0..len {
                    let int = reader.get_i32_be()?;
                    int_array.push(int);
                }
                Ok(Self::IntArray(int_array))
            }
            LONG_ARRAY_ID => {
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                let len = len as usize;
                let mut long_array = Vec::with_capacity(len);
                for _ in 0..len {
                    let long = reader.get_i64_be()?;
                    long_array.push(long);
                }
                Ok(Self::LongArray(long_array))
            }
            _ => Err(Error::UnknownTagId(tag_id)),
        }
    }

    #[must_use]
    pub fn extract_byte(&self) -> Option<i8> {
        match self {
            Self::Byte(byte) => Some(*byte),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_short(&self) -> Option<i16> {
        match self {
            Self::Short(short) => Some(*short),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_int(&self) -> Option<i32> {
        match self {
            Self::Int(int) => Some(*int),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_long(&self) -> Option<i64> {
        match self {
            Self::Long(long) => Some(*long),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_float(&self) -> Option<f32> {
        match self {
            Self::Float(float) => Some(*float),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_double(&self) -> Option<f64> {
        match self {
            Self::Double(double) => Some(*double),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_bool(&self) -> Option<bool> {
        match self {
            Self::Byte(byte) => Some(byte != &0),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_byte_array(&self) -> Option<&[u8]> {
        match self {
            Self::ByteArray(byte_array) => Some(byte_array),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_string(&self) -> Option<&str> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_list(&self) -> Option<&[Self]> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_compound(&self) -> Option<&NbtCompound> {
        match self {
            Self::Compound(compound) => Some(compound),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_int_array(&self) -> Option<&[i32]> {
        match self {
            Self::IntArray(int_array) => Some(int_array),
            _ => None,
        }
    }

    #[must_use]
    pub fn extract_long_array(&self) -> Option<&[i64]> {
        match self {
            Self::LongArray(long_array) => Some(long_array),
            _ => None,
        }
    }
}

impl From<&str> for NbtTag {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<&[u8]> for NbtTag {
    fn from(value: &[u8]) -> Self {
        let mut cloned = Vec::with_capacity(value.len());
        cloned.copy_from_slice(value);
        Self::ByteArray(cloned.into_boxed_slice())
    }
}

impl From<f32> for NbtTag {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<f64> for NbtTag {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<bool> for NbtTag {
    fn from(value: bool) -> Self {
        Self::Byte(i8::from(value))
    }
}

impl Serialize for NbtTag {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::End => serializer.serialize_unit(),
            Self::Byte(v) => serializer.serialize_i8(*v),
            Self::Short(v) => serializer.serialize_i16(*v),
            Self::Int(v) => serializer.serialize_i32(*v),
            Self::Long(v) => serializer.serialize_i64(*v),
            Self::Float(v) => serializer.serialize_f32(*v),
            Self::Double(v) => serializer.serialize_f64(*v),
            Self::ByteArray(v) => nbt_byte_array(v, serializer),
            Self::String(v) => serializer.serialize_str(v),
            Self::List(v) => {
                use serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Self::Compound(v) => v.serialize(serializer),
            Self::IntArray(v) => nbt_int_array(v, serializer),
            Self::LongArray(v) => nbt_long_array(v, serializer),
        }
    }
}

impl<'de> Deserialize<'de> for NbtTag {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct NbtTagVisitor;

        impl<'de> serde::de::Visitor<'de> for NbtTagVisitor {
            type Value = NbtTag;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an NBT tag")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(NbtTag::Byte(i8::from(v)))
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E> {
                Ok(NbtTag::Byte(v))
            }

            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E> {
                Ok(NbtTag::Short(v))
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E> {
                Ok(NbtTag::Int(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(NbtTag::Long(v))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E> {
                Ok(NbtTag::Float(v))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                Ok(NbtTag::Double(v))
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(NbtTag::String(v.to_string()))
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let curr = deserializer::take_curr_visitor_seq_list_id().unwrap_or(LIST_ID);

                match curr {
                    INT_ARRAY_ID => {
                        let mut vec = Vec::new();
                        while let Some(value) = seq.next_element()? {
                            vec.push(value);
                        }
                        Ok(NbtTag::IntArray(vec))
                    }
                    LONG_ARRAY_ID => {
                        let mut vec = Vec::new();
                        while let Some(value) = seq.next_element()? {
                            vec.push(value);
                        }
                        Ok(NbtTag::LongArray(vec))
                    }
                    BYTE_ARRAY_ID => {
                        let mut vec = Vec::new();
                        while let Some(value) = seq.next_element()? {
                            vec.push(value);
                        }
                        Ok(NbtTag::ByteArray(vec.into_boxed_slice()))
                    }
                    LIST_ID => {
                        let mut vec = Vec::new();
                        while let Some(value) = seq.next_element()? {
                            vec.push(value);
                        }
                        Ok(NbtTag::List(vec))
                    }
                    _ => Err(serde::de::Error::custom(format!(
                        "Invalid list type id: {curr}"
                    ))),
                }
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                Ok(NbtTag::Compound(NbtCompound::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?))
            }
        }

        deserializer.deserialize_any(NbtTagVisitor)
    }
}

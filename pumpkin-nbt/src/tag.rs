use compound::NbtCompound;
use deserializer::ReadAdaptor;
use io::Read;
use serializer::WriteAdaptor;

use crate::*;

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
    List(Box<[NbtTag]>) = LIST_ID,
    Compound(NbtCompound) = COMPOUND_ID,
    IntArray(Box<[i32]>) = INT_ARRAY_ID,
    LongArray(Box<[i64]>) = LONG_ARRAY_ID,
}

impl NbtTag {
    /// Returns the numeric id associated with the data type.
    pub const fn get_type_id(&self) -> u8 {
        // See https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        unsafe { *(self as *const Self as *const u8) }
    }

    pub fn serialize<W>(&self, w: &mut WriteAdaptor<W>) -> serializer::Result<()>
    where
        W: Write,
    {
        w.write_u8_be(self.get_type_id())?;
        self.serialize_data(w)?;
        Ok(())
    }

    pub fn serialize_data<W>(&self, w: &mut WriteAdaptor<W>) -> serializer::Result<()>
    where
        W: Write,
    {
        match self {
            NbtTag::End => {}
            NbtTag::Byte(byte) => w.write_i8_be(*byte)?,
            NbtTag::Short(short) => w.write_i16_be(*short)?,
            NbtTag::Int(int) => w.write_i32_be(*int)?,
            NbtTag::Long(long) => w.write_i64_be(*long)?,
            NbtTag::Float(float) => w.write_f32_be(*float)?,
            NbtTag::Double(double) => w.write_f64_be(*double)?,
            NbtTag::ByteArray(byte_array) => {
                let len = byte_array.len();
                if len > i32::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                w.write_i32_be(len as i32)?;
                w.write_slice(byte_array)?;
            }
            NbtTag::String(string) => {
                let java_string = cesu8::to_java_cesu8(string);
                let len = java_string.len();
                if len > u16::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                w.write_u16_be(len as u16)?;
                w.write_slice(&java_string)?;
            }
            NbtTag::List(list) => {
                let len = list.len();
                if len > i32::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                w.write_u8_be(list.first().unwrap_or(&NbtTag::End).get_type_id())?;
                w.write_i32_be(len as i32)?;
                for nbt_tag in list {
                    nbt_tag.serialize_data(w)?;
                }
            }
            NbtTag::Compound(compound) => {
                compound.serialize_content(w)?;
            }
            NbtTag::IntArray(int_array) => {
                let len = int_array.len();
                if len > i32::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                w.write_i32_be(len as i32)?;
                for int in int_array {
                    w.write_i32_be(*int)?;
                }
            }
            NbtTag::LongArray(long_array) => {
                let len = long_array.len();
                if len > i32::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                w.write_i32_be(len as i32)?;

                for long in long_array {
                    w.write_i64_be(*long)?;
                }
            }
        };
        Ok(())
    }

    pub fn deserialize<R>(reader: &mut ReadAdaptor<R>) -> Result<NbtTag, Error>
    where
        R: Read,
    {
        let tag_id = reader.get_u8_be()?;
        Self::deserialize_data(reader, tag_id)
    }

    pub fn skip_data<R>(reader: &mut ReadAdaptor<R>, tag_id: u8) -> Result<(), Error>
    where
        R: Read,
    {
        match tag_id {
            END_ID => Ok(()),
            BYTE_ID => reader.skip_bytes(1),
            SHORT_ID => reader.skip_bytes(2),
            INT_ID => reader.skip_bytes(4),
            LONG_ID => reader.skip_bytes(8),
            FLOAT_ID => reader.skip_bytes(4),
            DOUBLE_ID => reader.skip_bytes(8),
            BYTE_ARRAY_ID => {
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }
                reader.skip_bytes(len as u64)
            }
            STRING_ID => {
                let len = reader.get_u16_be()?;
                reader.skip_bytes(len as u64)
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

                reader.skip_bytes(len as u64 * 4)
            }
            LONG_ARRAY_ID => {
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                reader.skip_bytes(len as u64 * 8)
            }
            _ => Err(Error::UnknownTagId(tag_id)),
        }
    }

    pub fn deserialize_data<R>(reader: &mut ReadAdaptor<R>, tag_id: u8) -> Result<NbtTag, Error>
    where
        R: Read,
    {
        match tag_id {
            END_ID => Ok(NbtTag::End),
            BYTE_ID => {
                let byte = reader.get_i8_be()?;
                Ok(NbtTag::Byte(byte))
            }
            SHORT_ID => {
                let short = reader.get_i16_be()?;
                Ok(NbtTag::Short(short))
            }
            INT_ID => {
                let int = reader.get_i32_be()?;
                Ok(NbtTag::Int(int))
            }
            LONG_ID => {
                let long = reader.get_i64_be()?;
                Ok(NbtTag::Long(long))
            }
            FLOAT_ID => {
                let float = reader.get_f32_be()?;
                Ok(NbtTag::Float(float))
            }
            DOUBLE_ID => {
                let double = reader.get_f64_be()?;
                Ok(NbtTag::Double(double))
            }
            BYTE_ARRAY_ID => {
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                let byte_array = reader.read_boxed_slice(len as usize)?;
                Ok(NbtTag::ByteArray(byte_array))
            }
            STRING_ID => Ok(NbtTag::String(get_nbt_string(reader)?)),
            LIST_ID => {
                let tag_type_id = reader.get_u8_be()?;
                let len = reader.get_i32_be()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                let mut list = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let tag = NbtTag::deserialize_data(reader, tag_type_id)?;
                    assert_eq!(tag.get_type_id(), tag_type_id);
                    list.push(tag);
                }
                Ok(NbtTag::List(list.into_boxed_slice()))
            }
            COMPOUND_ID => Ok(NbtTag::Compound(NbtCompound::deserialize_content(reader)?)),
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
                Ok(NbtTag::IntArray(int_array.into_boxed_slice()))
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
                Ok(NbtTag::LongArray(long_array.into_boxed_slice()))
            }
            _ => Err(Error::UnknownTagId(tag_id)),
        }
    }

    pub fn extract_byte(&self) -> Option<i8> {
        match self {
            NbtTag::Byte(byte) => Some(*byte),
            _ => None,
        }
    }

    pub fn extract_short(&self) -> Option<i16> {
        match self {
            NbtTag::Short(short) => Some(*short),
            _ => None,
        }
    }

    pub fn extract_int(&self) -> Option<i32> {
        match self {
            NbtTag::Int(int) => Some(*int),
            _ => None,
        }
    }

    pub fn extract_long(&self) -> Option<i64> {
        match self {
            NbtTag::Long(long) => Some(*long),
            _ => None,
        }
    }

    pub fn extract_float(&self) -> Option<f32> {
        match self {
            NbtTag::Float(float) => Some(*float),
            _ => None,
        }
    }

    pub fn extract_double(&self) -> Option<f64> {
        match self {
            NbtTag::Double(double) => Some(*double),
            _ => None,
        }
    }

    pub fn extract_bool(&self) -> Option<bool> {
        match self {
            NbtTag::Byte(byte) => Some(*byte != 0),
            _ => None,
        }
    }

    pub fn extract_byte_array(&self) -> Option<Box<[u8]>> {
        match self {
            // Note: Bytes are free to clone, so we can hand out an owned type
            NbtTag::ByteArray(byte_array) => Some(byte_array.clone()),
            _ => None,
        }
    }

    pub fn extract_string(&self) -> Option<&String> {
        match self {
            NbtTag::String(string) => Some(string),
            _ => None,
        }
    }

    pub fn extract_list(&self) -> Option<&[NbtTag]> {
        match self {
            NbtTag::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn extract_compound(&self) -> Option<&NbtCompound> {
        match self {
            NbtTag::Compound(compound) => Some(compound),
            _ => None,
        }
    }

    pub fn extract_int_array(&self) -> Option<&[i32]> {
        match self {
            NbtTag::IntArray(int_array) => Some(int_array),
            _ => None,
        }
    }

    pub fn extract_long_array(&self) -> Option<&[i64]> {
        match self {
            NbtTag::LongArray(long_array) => Some(long_array),
            _ => None,
        }
    }
}

impl From<&str> for NbtTag {
    fn from(value: &str) -> Self {
        NbtTag::String(value.to_string())
    }
}

impl From<&[u8]> for NbtTag {
    fn from(value: &[u8]) -> Self {
        let mut cloned = Vec::with_capacity(value.len());
        cloned.copy_from_slice(value);
        NbtTag::ByteArray(cloned.into_boxed_slice())
    }
}

impl From<f32> for NbtTag {
    fn from(value: f32) -> Self {
        NbtTag::Float(value)
    }
}

impl From<f64> for NbtTag {
    fn from(value: f64) -> Self {
        NbtTag::Double(value)
    }
}

impl From<bool> for NbtTag {
    fn from(value: bool) -> Self {
        NbtTag::Byte(value as i8)
    }
}

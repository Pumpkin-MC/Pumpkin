//! The in-memory representation of individual NBT tags.

use compound::NbtCompound;
use deserializer::NbtReadHelper;
use serializer::NbtWriteHelper;

use crate::{
    BYTE_ARRAY_ID, BYTE_ID, COMPOUND_ID, DOUBLE_ID, END_ID, Error, FLOAT_ID, INT_ARRAY_ID, INT_ID,
    LIST_ID, LONG_ARRAY_ID, LONG_ID, SHORT_ID, STRING_ID, compound, deserializer, serializer,
};

/// A value represented by one of the tag types defined by the NBT format.
#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum NbtTag {
    /// Marks the end of a compound.
    End = END_ID,
    /// An 8-bit signed integer.
    Byte(i8) = BYTE_ID,
    /// A 16-bit signed integer.
    Short(i16) = SHORT_ID,
    /// A 32-bit signed integer.
    Int(i32) = INT_ID,
    /// A 64-bit signed integer.
    Long(i64) = LONG_ID,
    /// A 32-bit floating-point number.
    Float(f32) = FLOAT_ID,
    /// A 64-bit floating-point number.
    Double(f64) = DOUBLE_ID,
    /// An array of 8-bit signed integers.
    ByteArray(Box<[i8]>) = BYTE_ARRAY_ID,
    /// A string.
    String(Box<str>) = STRING_ID,
    /// A sequence of tags.
    List(Vec<Self>) = LIST_ID,
    /// A map of named tags.
    Compound(NbtCompound) = COMPOUND_ID,
    /// An array of 32-bit signed integers.
    IntArray(Vec<i32>) = INT_ARRAY_ID,
    /// An array of 64-bit signed integers.
    LongArray(Vec<i64>) = LONG_ARRAY_ID,
}

impl NbtTag {
    /// Returns the numeric id associated with the data type.
    #[must_use]
    pub const fn get_type_id(&self) -> u8 {
        // SAFETY: Since Self is repr(u8), it is guaranteed to hold the discriminant in the first byte
        // See https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        unsafe { *std::ptr::from_ref::<Self>(self).cast::<u8>() }
    }

    /// Serializes the tag's type ID followed by its payload.
    pub fn serialize<W: NbtWriteHelper>(self, w: &mut W) -> serializer::Result<()> {
        w.write_u8(self.get_type_id())?;
        self.serialize_data(w)?;
        Ok(())
    }

    /// Gets the element type of [`NbtTag::List`] the provided `Vec`
    /// represents. If any elements in the `Vec` are found to be of
    /// different types, this returns [`COMPOUND_ID`].
    #[must_use]
    fn get_list_element_type_id(list: &[Self]) -> u8 {
        let mut element_id = END_ID;

        for tag in list {
            let id = tag.get_type_id();
            if element_id == END_ID {
                element_id = id;
            } else if element_id != id {
                return COMPOUND_ID;
            }
        }

        element_id
    }

    /// Tries to unwrap (flatten) a wrapped `NbtTag`. If there is a wrapped tag, it is returned.
    /// If no unwrap is possible, this returns the given tag.
    fn flatten(tag: Self) -> Self {
        if let Self::Compound(mut compound) = tag {
            // Try to get the wrapped tag, stored by "".
            if Self::is_wrapper_compound(&compound) {
                compound
                    .child_tags
                    .remove("")
                    .unwrap_or(Self::Compound(compound))
            } else {
                Self::Compound(compound)
            }
        } else {
            tag
        }
    }

    /// Returns whether an [`NbtCompound`] is a wrapper compound.
    ///
    /// A *wrapper compound* is a compound that stores exactly one
    /// key-value pair, an empty string key (`""`) and an `NbtTag`.
    fn is_wrapper_compound(compound: &NbtCompound) -> bool {
        compound.child_tags.len() == 1 && compound.child_tags.contains_key("")
    }

    /// Wraps the provided tag if needed with the provided element type of list
    /// the wrapped tag, if any, would be added to.
    fn wrap_tag_if_needed(element_type: u8, tag: Self) -> Self {
        if element_type == COMPOUND_ID {
            if let Self::Compound(compound) = &tag
                && !Self::is_wrapper_compound(compound)
            {
                tag
            } else {
                Self::wrap_tag(tag)
            }
        } else {
            tag
        }
    }

    fn wrap_tag(tag: Self) -> Self {
        let mut compound = NbtCompound::new();
        compound.put("", tag);
        Self::Compound(compound)
    }

    /// Serializes the tag payload without writing its type ID.
    pub fn serialize_data<W: NbtWriteHelper>(self, w: &mut W) -> serializer::Result<()> {
        match self {
            Self::End => {}
            Self::Byte(byte) => w.write_i8(byte)?,
            Self::Short(short) => w.write_i16(short)?,
            Self::Int(int) => w.write_i32(int)?,
            Self::Long(long) => w.write_i64(long)?,
            Self::Float(float) => w.write_f32(float)?,
            Self::Double(double) => w.write_f64(double)?,
            Self::ByteArray(byte_array) => {
                let len = byte_array.len();
                if len > i32::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                w.write_i32(len as i32)?;
                for int in byte_array {
                    w.write_i8(int)?;
                }
            }
            Self::String(string) => {
                w.write_string(&string)?;
            }
            Self::List(list) => {
                let len = list.len();
                if len > i32::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                let list_element_id = Self::get_list_element_type_id(&list);

                w.write_u8(list_element_id)?;
                w.write_i32(len as i32)?;
                for nbt_tag in list {
                    // Since tags in the same list tag must have the same type,
                    // we need to handle those of different tag types by
                    // wrapping them in `NbtCompound`s if needed.
                    Self::wrap_tag_if_needed(list_element_id, nbt_tag).serialize_data(w)?;
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

                w.write_i32(len as i32)?;
                for int in int_array {
                    w.write_i32(int)?;
                }
            }
            Self::LongArray(long_array) => {
                let len = long_array.len();
                if len > i32::MAX as usize {
                    return Err(Error::LargeLength(len));
                }

                w.write_i32(len as i32)?;
                for long in long_array {
                    w.write_i64(long)?;
                }
            }
        }
        Ok(())
    }

    /// Deserializes a type ID and its following payload.
    pub fn deserialize<'a, R: NbtReadHelper<'a>>(reader: &mut R) -> Result<Self, Error> {
        let tag_id = reader.get_u8()?;
        Self::deserialize_data(reader, tag_id)
    }

    /// Advances a reader past the payload belonging to `tag_id`.
    pub fn skip_data<'a, R: NbtReadHelper<'a>>(reader: &mut R, tag_id: u8) -> Result<(), Error> {
        Self::skip_data_depth(reader, tag_id, 0)
    }

    /// Advances a reader past the payload belonging to `tag_id` with depth tracking.
    pub fn skip_data_depth<'a, R: NbtReadHelper<'a>>(
        reader: &mut R,
        tag_id: u8,
        depth: usize,
    ) -> Result<(), Error> {
        if depth > crate::MAX_NBT_DEPTH {
            return Err(Error::MaxDepthExceeded);
        }

        match tag_id {
            END_ID => Ok(()),
            BYTE_ID => reader.skip_i8(),
            SHORT_ID => reader.skip_i16(),
            INT_ID => reader.skip_i32(),
            LONG_ID => reader.skip_i64(),
            FLOAT_ID => reader.skip_f32(),
            DOUBLE_ID => reader.skip_f64(),
            BYTE_ARRAY_ID => {
                let len = reader.get_i32()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }
                let len = len as usize;
                if len > crate::MAX_ARRAY_LENGTH {
                    return Err(Error::LargeLength(len));
                }
                reader.skip_bytes(len as i64)
            }
            STRING_ID => reader.skip_string(),
            LIST_ID => {
                let tag_type_id = reader.get_u8()?;
                let len = reader.get_i32()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }
                if tag_type_id == END_ID && len > 0 {
                    return Err(Error::InvalidListTag(tag_type_id));
                }

                let len = len as usize;
                if len > crate::MAX_ARRAY_LENGTH {
                    return Err(Error::LargeLength(len));
                }

                for _ in 0..len {
                    Self::skip_data_depth(reader, tag_type_id, depth + 1)?;
                }

                Ok(())
            }
            COMPOUND_ID => NbtCompound::skip_content_depth(reader, depth + 1),
            INT_ARRAY_ID => {
                let len = reader.get_i32()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                let len = len as usize;
                if len > crate::MAX_ARRAY_LENGTH {
                    return Err(Error::LargeLength(len));
                }

                for _ in 0..len {
                    reader.skip_i32()?;
                }

                Ok(())
            }
            LONG_ARRAY_ID => {
                let len = reader.get_i32()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                let len = len as usize;
                if len > crate::MAX_ARRAY_LENGTH {
                    return Err(Error::LargeLength(len));
                }

                for _ in 0..len {
                    reader.skip_i64()?;
                }

                Ok(())
            }
            _ => Err(Error::UnknownTagId(tag_id)),
        }
    }

    /// Deserializes a payload whose type is identified by `tag_id`.
    pub fn deserialize_data<'a, R: NbtReadHelper<'a>>(
        reader: &mut R,
        tag_id: u8,
    ) -> Result<Self, Error> {
        Self::deserialize_data_depth(reader, tag_id, 0)
    }

    /// Deserializes a payload whose type is identified by `tag_id` with depth tracking.
    #[allow(clippy::too_many_lines)]
    pub fn deserialize_data_depth<'a, R: NbtReadHelper<'a>>(
        reader: &mut R,
        tag_id: u8,
        depth: usize,
    ) -> Result<Self, Error> {
        if depth > crate::MAX_NBT_DEPTH {
            return Err(Error::MaxDepthExceeded);
        }

        match tag_id {
            END_ID => Ok(Self::End),
            BYTE_ID => {
                let byte = reader.get_i8()?;
                Ok(Self::Byte(byte))
            }
            SHORT_ID => {
                let short = reader.get_i16()?;
                Ok(Self::Short(short))
            }
            INT_ID => {
                let int = reader.get_i32()?;
                Ok(Self::Int(int))
            }
            LONG_ID => {
                let long = reader.get_i64()?;
                Ok(Self::Long(long))
            }
            FLOAT_ID => {
                let float = reader.get_f32()?;
                Ok(Self::Float(float))
            }
            DOUBLE_ID => {
                let double = reader.get_f64()?;
                Ok(Self::Double(double))
            }
            BYTE_ARRAY_ID => {
                let len = reader.get_i32()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                let len = len as usize;
                if len > crate::MAX_ARRAY_LENGTH {
                    return Err(Error::LargeLength(len));
                }
                let mut byte_array = Vec::with_capacity(len.min(4096));
                for _ in 0..len {
                    let byte = reader.get_i8()?;
                    byte_array.push(byte);
                }
                Ok(Self::ByteArray(byte_array.into()))
            }
            STRING_ID => Ok(Self::String(reader.get_string()?.into_owned().into())),
            LIST_ID => {
                let tag_type_id = reader.get_u8()?;
                let len = reader.get_i32()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }
                if tag_type_id == END_ID && len > 0 {
                    return Err(Error::InvalidListTag(tag_type_id));
                }

                let len = len as usize;
                if len > crate::MAX_ARRAY_LENGTH {
                    return Err(Error::LargeLength(len));
                }

                let mut list = Vec::with_capacity(len.min(4096));
                for _ in 0..len {
                    let tag = Self::deserialize_data_depth(reader, tag_type_id, depth + 1)?;
                    if tag.get_type_id() != tag_type_id {
                        return Err(Error::InvalidListTag(tag.get_type_id()));
                    }
                    // Try unwrapping the tag.
                    list.push(Self::flatten(tag));
                }
                Ok(Self::List(list))
            }
            COMPOUND_ID => Ok(Self::Compound(NbtCompound::deserialize_content_depth(
                reader,
                depth + 1,
            )?)),
            INT_ARRAY_ID => {
                let len = reader.get_i32()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                let len = len as usize;
                if len > crate::MAX_ARRAY_LENGTH {
                    return Err(Error::LargeLength(len));
                }

                let mut int_array = Vec::with_capacity(len.min(4096));
                for _ in 0..len {
                    let int = reader.get_i32()?;
                    int_array.push(int);
                }
                Ok(Self::IntArray(int_array))
            }
            LONG_ARRAY_ID => {
                let len = reader.get_i32()?;
                if len < 0 {
                    return Err(Error::NegativeLength(len));
                }

                let len = len as usize;
                if len > crate::MAX_ARRAY_LENGTH {
                    return Err(Error::LargeLength(len));
                }

                let mut long_array = Vec::with_capacity(len.min(4096));
                for _ in 0..len {
                    let long = reader.get_i64()?;
                    long_array.push(long);
                }
                Ok(Self::LongArray(long_array))
            }
            _ => Err(Error::UnknownTagId(tag_id)),
        }
    }

    /// Returns the contained byte, if this is a byte tag.
    #[must_use]
    pub const fn extract_byte(&self) -> Option<i8> {
        match self {
            Self::Byte(byte) => Some(*byte),
            _ => None,
        }
    }

    /// Returns the contained short, if this is a short tag.
    #[must_use]
    pub const fn extract_short(&self) -> Option<i16> {
        match self {
            Self::Short(short) => Some(*short),
            _ => None,
        }
    }

    /// Returns the contained integer, if this is an integer tag.
    #[must_use]
    pub const fn extract_int(&self) -> Option<i32> {
        match self {
            Self::Int(int) => Some(*int),
            _ => None,
        }
    }

    /// Returns the contained long, if this is a long tag.
    #[must_use]
    pub const fn extract_long(&self) -> Option<i64> {
        match self {
            Self::Long(long) => Some(*long),
            _ => None,
        }
    }

    /// Returns the contained float, if this is a float tag.
    #[must_use]
    pub const fn extract_float(&self) -> Option<f32> {
        match self {
            Self::Float(float) => Some(*float),
            _ => None,
        }
    }

    /// Returns the contained double, if this is a double tag.
    #[must_use]
    pub const fn extract_double(&self) -> Option<f64> {
        match self {
            Self::Double(double) => Some(*double),
            _ => None,
        }
    }

    /// Returns the contained byte as a boolean, where zero is `false`.
    #[must_use]
    pub fn extract_bool(&self) -> Option<bool> {
        match self {
            Self::Byte(byte) => Some(byte != &0),
            _ => None,
        }
    }

    /// Returns the contained byte array, if this is a byte-array tag.
    #[must_use]
    pub fn extract_byte_array(&self) -> Option<&[i8]> {
        match self {
            Self::ByteArray(byte_array) => Some(byte_array),
            _ => None,
        }
    }

    /// Returns the contained string, if this is a string tag.
    #[must_use]
    pub fn extract_string(&self) -> Option<&str> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }

    /// Returns the contained list, if this is a list tag.
    #[must_use]
    pub fn extract_list(&self) -> Option<&[Self]> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }

    /// Returns the contained compound, if this is a compound tag.
    #[must_use]
    pub const fn extract_compound(&self) -> Option<&NbtCompound> {
        match self {
            Self::Compound(compound) => Some(compound),
            _ => None,
        }
    }

    /// Returns the contained integer array, if this is an integer-array tag.
    #[must_use]
    pub fn extract_int_array(&self) -> Option<&[i32]> {
        match self {
            Self::IntArray(int_array) => Some(int_array),
            _ => None,
        }
    }

    /// Returns the contained long array, if this is a long-array tag.
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
        Self::String(value.into())
    }
}

impl From<&[i8]> for NbtTag {
    fn from(value: &[i8]) -> Self {
        Self::ByteArray(value.into())
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

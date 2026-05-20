use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::deserializer::NbtReadHelper;
use crate::serializer::NbtWriteHelper;
use crate::tag::NbtTag;
use crate::{END_ID, Error, Nbt};
use std::collections::hash_map::IntoIter;
use std::io::ErrorKind;

#[macro_export]
macro_rules! nbt_compound_tag {
    { $($key:literal : $tag:expr),+ $(,)* } => {
        {
            let mut compound = NbtCompound::new();
            $( compound.put($key, $tag); )+
            NbtTag::Compound(compound)
        }
    };
    // For empty compounds
    {} => {
        NbtTag::Compound(NbtCompound::new())
    };
}

/// Represents a Compound NBT tag, effectively a hash map.
///
/// Internally, this uses a `HashMap<String, NbtTag>`, which does not preserve insertion order,
/// just like Minecraft: Java Edition, but it does mean lookups are O(1).
///
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NbtCompound {
    pub child_tags: HashMap<Box<str>, NbtTag>,
}

impl NbtCompound {
    #[must_use]
    pub fn new() -> Self {
        Self {
            child_tags: HashMap::new(),
        }
    }

    pub fn skip_content<R: NbtReadHelper>(reader: &mut R) -> Result<(), Error> {
        loop {
            let tag_id = match reader.get_u8() {
                Ok(id) => id,
                Err(Error::Incomplete(e)) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            if tag_id == END_ID {
                break;
            }

            reader.skip_string()?;

            // Skip Value
            NbtTag::skip_data(reader, tag_id)?;
        }

        Ok(())
    }

    pub fn deserialize_content<R: NbtReadHelper>(reader: &mut R) -> Result<Self, Error> {
        let mut compound = Self::new();

        loop {
            let tag_id = match reader.get_u8() {
                Ok(id) => id,
                Err(Error::Incomplete(e)) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            if tag_id == END_ID {
                break;
            }

            let name = reader.get_string()?;
            let tag = NbtTag::deserialize_data(reader, tag_id)?;

            compound.child_tags.insert(name.into(), tag);
        }

        Ok(compound)
    }

    pub fn serialize_content<W: NbtWriteHelper>(self, w: &mut W) -> Result<(), Error> {
        for (name, tag) in self.child_tags {
            w.write_u8(tag.get_type_id())?;
            w.write_string(&name)?;
            tag.serialize_data(w)?;
        }
        w.write_u8(END_ID)?;
        Ok(())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.child_tags.is_empty()
    }

    pub fn put(&mut self, name: &str, value: impl Into<NbtTag>) {
        if !self.child_tags.contains_key(name) {
            self.child_tags.insert(name.into(), value.into());
        }
    }

    pub fn put_string(&mut self, name: &str, value: String) {
        self.put(name, NbtTag::String(value.into()));
    }

    pub fn put_list(&mut self, name: &str, value: Vec<NbtTag>) {
        self.put(name, NbtTag::List(value));
    }

    pub fn put_byte(&mut self, name: &str, value: i8) {
        self.put(name, NbtTag::Byte(value));
    }

    pub fn put_bool(&mut self, name: &str, value: bool) {
        self.put(name, NbtTag::Byte(i8::from(value)));
    }

    pub fn put_short(&mut self, name: &str, value: i16) {
        self.put(name, NbtTag::Short(value));
    }

    pub fn put_int(&mut self, name: &str, value: i32) {
        self.put(name, NbtTag::Int(value));
    }
    pub fn put_long(&mut self, name: &str, value: i64) {
        self.put(name, NbtTag::Long(value));
    }

    pub fn put_float(&mut self, name: &str, value: f32) {
        self.put(name, NbtTag::Float(value));
    }

    pub fn put_double(&mut self, name: &str, value: f64) {
        self.put(name, NbtTag::Double(value));
    }

    pub fn put_compound(&mut self, name: &str, value: Self) {
        self.put(name, NbtTag::Compound(value));
    }

    #[must_use]
    pub fn get_byte(&self, name: &str) -> Option<i8> {
        self.get(name).and_then(super::tag::NbtTag::extract_byte)
    }

    #[inline]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&NbtTag> {
        self.child_tags.get(name)
    }

    #[inline]
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.child_tags.contains_key(name)
    }

    #[must_use]
    pub fn get_short(&self, name: &str) -> Option<i16> {
        self.get(name).and_then(super::tag::NbtTag::extract_short)
    }

    #[must_use]
    pub fn get_int(&self, name: &str) -> Option<i32> {
        self.get(name).and_then(super::tag::NbtTag::extract_int)
    }

    #[must_use]
    pub fn get_long(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(super::tag::NbtTag::extract_long)
    }

    #[must_use]
    pub fn get_float(&self, name: &str) -> Option<f32> {
        self.get(name).and_then(super::tag::NbtTag::extract_float)
    }

    #[must_use]
    pub fn get_double(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(super::tag::NbtTag::extract_double)
    }

    #[must_use]
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name).and_then(super::tag::NbtTag::extract_bool)
    }

    #[must_use]
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(|tag| tag.extract_string())
    }

    #[must_use]
    pub fn get_list(&self, name: &str) -> Option<&[NbtTag]> {
        self.get(name).and_then(|tag| tag.extract_list())
    }

    #[must_use]
    pub fn get_compound(&self, name: &str) -> Option<&Self> {
        self.get(name).and_then(|tag| tag.extract_compound())
    }

    #[must_use]
    pub fn get_int_array(&self, name: &str) -> Option<&[i32]> {
        self.get(name).and_then(|tag| tag.extract_int_array())
    }

    #[must_use]
    pub fn get_long_array(&self, name: &str) -> Option<&[i64]> {
        self.get(name).and_then(|tag| tag.extract_long_array())
    }
}

impl From<Nbt> for NbtCompound {
    fn from(value: Nbt) -> Self {
        value.root_tag
    }
}

impl FromIterator<(String, NbtTag)> for NbtCompound {
    fn from_iter<T: IntoIterator<Item = (String, NbtTag)>>(iter: T) -> Self {
        let mut compound = Self::new();
        for (key, value) in iter {
            compound.put(&key, value);
        }
        compound
    }
}

impl FromIterator<(Box<str>, NbtTag)> for NbtCompound {
    fn from_iter<T: IntoIterator<Item = (Box<str>, NbtTag)>>(iter: T) -> Self {
        let mut compound = Self::new();
        for (key, value) in iter {
            compound.child_tags.insert(key, value);
        }
        compound
    }
}

impl IntoIterator for NbtCompound {
    type Item = (Box<str>, NbtTag);
    type IntoIter = IntoIter<Box<str>, NbtTag>;

    fn into_iter(self) -> Self::IntoIter {
        self.child_tags.into_iter()
    }
}

impl Extend<(String, NbtTag)> for NbtCompound {
    fn extend<T: IntoIterator<Item = (String, NbtTag)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.put(&key, value);
        }
    }
}

impl Extend<(Box<str>, NbtTag)> for NbtCompound {
    fn extend<T: IntoIterator<Item = (Box<str>, NbtTag)>>(&mut self, iter: T) {
        self.child_tags.extend(iter);
    }
}

// Rust's AsRef is currently not reflexive so we need to implement it manually
impl AsRef<Self> for NbtCompound {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Serialize for NbtCompound {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.child_tags.len()))?;
        for (key, value) in &self.child_tags {
            map.serialize_entry(key, &value)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for NbtCompound {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CompoundVisitor;

        impl<'de> serde::de::Visitor<'de> for CompoundVisitor {
            type Value = NbtCompound;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an NBT compound")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let mut compound = NbtCompound::new();
                while let Some((key, value)) = map.next_entry::<Box<str>, NbtTag>()? {
                    compound.child_tags.insert(key, value);
                }
                Ok(compound)
            }
        }

        deserializer.deserialize_map(CompoundVisitor)
    }
}

impl From<NbtCompound> for NbtTag {
    fn from(value: NbtCompound) -> Self {
        Self::Compound(value)
    }
}

/// SNBT display implementation for `NbtCompound`
impl Display for NbtCompound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{")?;
        for (i, (key, value)) in self.child_tags.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{key}: {value}")?;
        }
        f.write_str("}")
    }
}

impl Display for NbtTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::End => Ok(()),
            Self::Byte(v) => write!(f, "{v}b"),
            Self::Short(v) => write!(f, "{v}s"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Long(v) => write!(f, "{v}L"),
            Self::Float(v) => write!(f, "{v}f"),
            Self::Double(v) => write!(f, "{v}d"),
            Self::String(v) => write!(f, "\"{v}\""), // TODO: Proper escaping needed for robust SNBT
            Self::Compound(v) => write!(f, "{v}"),
            Self::ByteArray(v) => {
                f.write_str("[B;")?;
                for (i, byte) in v.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, " {byte}b")?;
                }
                f.write_str("]")
            }
            Self::List(v) => {
                f.write_str("[")?;
                for (i, tag) in v.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{tag}")?;
                }
                f.write_str("]")
            }
            Self::IntArray(v) => {
                f.write_str("[I;")?;
                for (i, int) in v.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, " {int}")?;
                }
                f.write_str("]")
            }
            Self::LongArray(v) => {
                f.write_str("[L;")?;
                for (i, long) in v.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, " {long}L")?;
                }
                f.write_str("]")
            }
        }
    }
}

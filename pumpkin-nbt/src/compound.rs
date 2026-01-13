use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::deserializer::NbtReadHelper;
use crate::serializer::WriteAdaptor;
use crate::tag::NbtTag;
use crate::{END_ID, Error, Nbt, get_nbt_string};
use std::io::{ErrorKind, Read, Seek, Write};
use std::vec::IntoIter;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct NbtCompound {
    pub child_tags: Vec<(String, NbtTag)>,
}

impl NbtCompound {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            child_tags: Vec::new(),
        }
    }

    pub fn skip_content<R: Read + Seek>(reader: &mut NbtReadHelper<R>) -> Result<(), Error> {
        loop {
            let tag_id = match reader.get_u8_be() {
                Ok(id) => id,
                Err(err) => match err {
                    Error::Incomplete(err) => match err.kind() {
                        ErrorKind::UnexpectedEof => {
                            break;
                        }
                        _ => {
                            return Err(Error::Incomplete(err));
                        }
                    },
                    _ => {
                        return Err(err);
                    }
                },
            };
            if tag_id == END_ID {
                break;
            }

            let len = reader.get_u16_be()?;
            reader.skip_bytes(i64::from(len))?;

            NbtTag::skip_data(reader, tag_id)?;
        }

        Ok(())
    }

    pub fn deserialize_content<R: Read + Seek>(
        reader: &mut NbtReadHelper<R>,
    ) -> Result<Self, Error> {
        let mut compound = Self::new();

        loop {
            let tag_id = match reader.get_u8_be() {
                Ok(id) => id,
                Err(err) => match err {
                    Error::Incomplete(err) => match err.kind() {
                        ErrorKind::UnexpectedEof => {
                            break;
                        }
                        _ => {
                            return Err(Error::Incomplete(err));
                        }
                    },
                    _ => {
                        return Err(err);
                    }
                },
            };
            if tag_id == END_ID {
                break;
            }

            let name = get_nbt_string(reader)?;
            let tag = NbtTag::deserialize_data(reader, tag_id)?;
            compound.put(&name, tag);
        }

        Ok(compound)
    }

    pub fn serialize_content<W: Write>(&self, w: &mut WriteAdaptor<W>) -> Result<(), Error> {
        for (name, tag) in &self.child_tags {
            w.write_u8_be(tag.get_type_id())?;
            NbtTag::String(name.clone()).serialize_data(w)?;
            tag.serialize_data(w)?;
        }
        w.write_u8_be(END_ID)?;
        Ok(())
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.child_tags.is_empty()
    }

    pub fn put(&mut self, name: &str, value: impl Into<NbtTag>) {
        let name = name.to_owned();
        if !self.child_tags.iter().any(|(key, _)| key == &name) {
            self.child_tags.push((name, value.into()));
        }
    }

    pub fn put_string(&mut self, name: &str, value: String) {
        self.put(name, NbtTag::String(value));
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

    pub fn put_component(&mut self, name: &str, value: Self) {
        self.put(name, NbtTag::Compound(value));
    }

    #[must_use]
    pub fn get_byte(&self, name: &str) -> Option<i8> {
        self.get(name).and_then(super::tag::NbtTag::extract_byte)
    }

    #[inline]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&NbtTag> {
        self.child_tags
            .iter()
            .find(|k| k.0.as_str() == name)
            .map(|r| &r.1)
    }

    #[must_use]
    pub fn get_short(&self, name: &str) -> Option<i16> {
        self.get(name).and_then(|tag| tag.extract_short())
    }

    #[must_use]
    pub fn get_int(&self, name: &str) -> Option<i32> {
        self.get(name).and_then(|tag| tag.extract_int())
    }

    #[must_use]
    pub fn get_long(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(|tag| tag.extract_long())
    }

    #[must_use]
    pub fn get_float(&self, name: &str) -> Option<f32> {
        self.get(name).and_then(|tag| tag.extract_float())
    }

    #[must_use]
    pub fn get_double(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(|tag| tag.extract_double())
    }

    #[must_use]
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name).and_then(|tag| tag.extract_bool())
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
    pub fn get_compound(&self, name: &str) -> Option<&NbtCompound> {
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
        let mut compound = NbtCompound::new();
        for (key, value) in iter {
            compound.put(&key, value);
        }
        compound
    }
}

impl IntoIterator for NbtCompound {
    type Item = (String, NbtTag);
    type IntoIter = IntoIter<(String, NbtTag)>;

    fn into_iter(self) -> Self::IntoIter {
        self.child_tags.into_iter()
    }
}

impl Extend<(String, NbtTag)> for NbtCompound {
    fn extend<T: IntoIterator<Item = (String, NbtTag)>>(&mut self, iter: T) {
        self.child_tags.extend(iter)
    }
}

// Rust's AsRef is currently not reflexive so we need to implement it manually
impl AsRef<NbtCompound> for NbtCompound {
    fn as_ref(&self) -> &NbtCompound {
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
                while let Some((key, value)) = map.next_entry::<String, NbtTag>()? {
                    compound.put(&key, value);
                }
                Ok(compound)
            }
        }

        deserializer.deserialize_map(CompoundVisitor)
    }
}

impl From<NbtCompound> for NbtTag {
    fn from(value: NbtCompound) -> Self {
        NbtTag::Compound(value)
    }
}

/// SNBT display implementation for NbtCompound
impl Display for NbtCompound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{")?;
        for (i, (key, value)) in self.child_tags.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            f.write_str(&format!("{key}: {value}"))?;
        }
        f.write_str("}")
    }
}

impl Display for NbtTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NbtTag::End => Ok(()),
            NbtTag::Byte(value) => f.write_fmt(format_args!("{value}")),
            NbtTag::Short(value) => f.write_fmt(format_args!("{value}")),
            NbtTag::Int(value) => f.write_fmt(format_args!("{value}")),
            NbtTag::Long(value) => f.write_fmt(format_args!("{value}")),
            NbtTag::Float(value) => f.write_fmt(format_args!("{value}")),
            NbtTag::Double(value) => f.write_fmt(format_args!("{value}")),
            NbtTag::String(value) => f.write_fmt(format_args!("\"{value}\"")),
            NbtTag::Compound(value) => f.write_fmt(format_args!("{value}")),
            NbtTag::ByteArray(value) => {
                f.write_str("[B; ")?;
                for (i, byte) in value.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    f.write_fmt(format_args!("{byte}"))?;
                }
                f.write_str("]")
            }
            NbtTag::List(value) => {
                f.write_str("[")?;
                for (i, tag) in value.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    f.write_fmt(format_args!("{tag}"))?;
                }
                f.write_str("]")
            }
            NbtTag::IntArray(value) => {
                f.write_str("[I; ")?;
                for (i, int) in value.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    f.write_fmt(format_args!("{int}"))?;
                }
                f.write_str("]")
            }
            NbtTag::LongArray(value) => {
                f.write_str("[L; ")?;
                for (i, long) in value.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    f.write_fmt(format_args!("{long}"))?;
                }
                f.write_str("]")
            }
        }
    }
}

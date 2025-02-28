use crate::deserializer::ReadAdaptor;
use crate::serializer::WriteAdaptor;
use crate::tag::NbtTag;
use crate::{END_ID, Error, Nbt, get_nbt_string};
use std::io::{ErrorKind, Read, Write};
use std::vec::IntoIter;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct NbtCompound {
    pub child_tags: Vec<(String, NbtTag)>,
}

impl NbtCompound {
    pub fn new() -> NbtCompound {
        NbtCompound {
            child_tags: Vec::new(),
        }
    }

    pub fn skip_content<R>(reader: &mut ReadAdaptor<R>) -> Result<(), Error>
    where
        R: Read,
    {
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
            reader.skip_bytes(len as u64)?;

            NbtTag::skip_data(reader, tag_id)?;
        }

        Ok(())
    }

    pub fn deserialize_content<R>(reader: &mut ReadAdaptor<R>) -> Result<NbtCompound, Error>
    where
        R: Read,
    {
        let mut compound = NbtCompound::new();

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

    pub fn serialize_content<W>(&self, w: &mut WriteAdaptor<W>) -> Result<(), Error>
    where
        W: Write,
    {
        for (name, tag) in &self.child_tags {
            w.write_u8_be(tag.get_type_id())?;
            NbtTag::String(name.clone()).serialize_data(w)?;
            tag.serialize_data(w)?;
        }
        w.write_u8_be(END_ID)?;
        Ok(())
    }

    pub fn put(&mut self, name: &str, value: impl Into<NbtTag>) {
        let name = name.to_string();
        if !self.child_tags.iter().any(|(key, _)| key == &name) {
            self.child_tags.push((name, value.into()));
        }
    }

    pub fn put_byte(&mut self, name: &str, value: i8) {
        self.put(name, NbtTag::Byte(value));
    }

    pub fn put_bool(&mut self, name: &str, value: bool) {
        self.put(name, NbtTag::Byte(if value { 1 } else { 0 }));
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

    pub fn put_component(&mut self, name: &str, value: NbtCompound) {
        self.put(name, NbtTag::Compound(value));
    }

    pub fn get_byte(&self, name: &str) -> Option<i8> {
        self.get(name).and_then(|tag| tag.extract_byte())
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<&NbtTag> {
        for (key, value) in &self.child_tags {
            if key.as_str() == name {
                return Some(value);
            }
        }
        None
    }

    pub fn get_short(&self, name: &str) -> Option<i16> {
        self.get(name).and_then(|tag| tag.extract_short())
    }

    pub fn get_int(&self, name: &str) -> Option<i32> {
        self.get(name).and_then(|tag| tag.extract_int())
    }

    pub fn get_long(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(|tag| tag.extract_long())
    }

    pub fn get_float(&self, name: &str) -> Option<f32> {
        self.get(name).and_then(|tag| tag.extract_float())
    }

    pub fn get_double(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(|tag| tag.extract_double())
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name).and_then(|tag| tag.extract_bool())
    }

    pub fn get_string(&self, name: &str) -> Option<&String> {
        self.get(name).and_then(|tag| tag.extract_string())
    }

    pub fn get_list(&self, name: &str) -> Option<&[NbtTag]> {
        self.get(name).and_then(|tag| tag.extract_list())
    }

    pub fn get_compound(&self, name: &str) -> Option<&NbtCompound> {
        self.get(name).and_then(|tag| tag.extract_compound())
    }

    pub fn get_int_array(&self, name: &str) -> Option<&[i32]> {
        self.get(name).and_then(|tag| tag.extract_int_array())
    }

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

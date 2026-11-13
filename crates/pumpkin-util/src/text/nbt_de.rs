use pumpkin_nbt::tag::NbtTag;
use serde::de::value::{Error, SeqDeserializer, StrDeserializer};
use serde::de::{DeserializeSeed, Deserializer, IntoDeserializer, MapAccess, SeqAccess, Visitor};

/// Reads a `Deserialize` type straight from an NBT tag.
pub struct TagDeserializer<'a>(pub &'a NbtTag);

impl<'de> Deserializer<'de> for TagDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            NbtTag::End => visitor.visit_unit(),
            NbtTag::Byte(v) => visitor.visit_i8(*v),
            NbtTag::Short(v) => visitor.visit_i16(*v),
            NbtTag::Int(v) => visitor.visit_i32(*v),
            NbtTag::Long(v) => visitor.visit_i64(*v),
            NbtTag::Float(v) => visitor.visit_f32(*v),
            NbtTag::Double(v) => visitor.visit_f64(*v),
            NbtTag::String(v) => visitor.visit_borrowed_str(v),
            NbtTag::List(list) => visitor.visit_seq(TagSeq(list.iter())),
            NbtTag::Compound(compound) => visitor.visit_map(TagMap {
                entries: compound.child_tags.iter(),
                value: None,
            }),
            NbtTag::ByteArray(v) => visitor.visit_seq(SeqDeserializer::new(v.iter().copied())),
            NbtTag::IntArray(v) => visitor.visit_seq(SeqDeserializer::new(v.iter().copied())),
            NbtTag::LongArray(v) => visitor.visit_seq(SeqDeserializer::new(v.iter().copied())),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.0 {
            NbtTag::String(v) => visitor.visit_enum(StrDeserializer::<Error>::new(v)),
            _ => self.deserialize_any(visitor),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct
        identifier ignored_any
    }
}

struct TagSeq<'a>(std::slice::Iter<'a, NbtTag>);

impl<'de> SeqAccess<'de> for TagSeq<'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        self.0
            .next()
            .map(|tag| seed.deserialize(TagDeserializer(tag)))
            .transpose()
    }
}

struct TagMap<'a> {
    entries: std::collections::hash_map::Iter<'a, Box<str>, NbtTag>,
    value: Option<&'a NbtTag>,
}

impl<'de> MapAccess<'de> for TagMap<'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        let Some((key, value)) = self.entries.next() else {
            return Ok(None);
        };
        self.value = Some(value);
        seed.deserialize(key.as_ref().into_deserializer()).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        let value = self
            .value
            .take()
            .ok_or_else(|| serde::de::Error::custom("value without a key"))?;
        seed.deserialize(TagDeserializer(value))
    }
}

#[cfg(test)]
mod tests {
    use super::TagDeserializer;
    use pumpkin_nbt::tag::NbtTag;
    use serde::Deserialize;

    #[test]
    fn a_present_tag_is_some() {
        let tag = NbtTag::String("x".into());
        let value = Option::<String>::deserialize(TagDeserializer(&tag));
        assert_eq!(value, Ok(Some("x".to_string())));
    }
}

use serde::de::{self, Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TagType {
    Item(String),
    Tag(String),
}

impl TagType {
    #[allow(dead_code)]
    pub fn serialize(&self) -> String {
        match self {
            TagType::Item(name) => name.clone(),
            TagType::Tag(tag) => format!("#{tag}"),
        }
    }
}

pub struct TagVisitor;
impl Visitor<'_> for TagVisitor {
    type Value = TagType;
    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "valid tag")
    }
    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        match v.strip_prefix('#') {
            Some(v) => Ok(TagType::Tag(v.to_string())),
            None => Ok(TagType::Item(v.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for TagType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(TagVisitor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum RegistryEntryList {
    Single(TagType),
    Many(Vec<TagType>),
}

impl RegistryEntryList {
    pub fn into_vec(self) -> Vec<TagType> {
        match self {
            RegistryEntryList::Single(s) => vec![s],
            RegistryEntryList::Many(s) => s,
        }
    }
}

impl PartialEq<TagType> for RegistryEntryList {
    fn eq(&self, other: &TagType) -> bool {
        match self {
            RegistryEntryList::Single(ingredient) => other == ingredient,
            RegistryEntryList::Many(ingredients) => ingredients.contains(other),
        }
    }
}

impl<'de> Deserialize<'de> for RegistryEntryList {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SlotTypeVisitor;
        impl<'de> Visitor<'de> for SlotTypeVisitor {
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "valid ingredient slot")
            }

            type Value = RegistryEntryList;

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(RegistryEntryList::Single(TagVisitor.visit_str(v)?))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut ingredients: Vec<TagType> = vec![];
                while let Some(element) = seq.next_element()? {
                    ingredients.push(element)
                }
                if ingredients.len() == 1 {
                    Ok(RegistryEntryList::Single(ingredients[0].clone()))
                } else {
                    Ok(RegistryEntryList::Many(ingredients))
                }
            }
        }
        deserializer.deserialize_any(SlotTypeVisitor)
    }
}

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
            TagType::Tag(tag) => format!("#{}", tag),
        }
    }
}

pub struct TagVisitor;
impl Visitor<'_> for TagVisitor {
    type Value = TagType;
    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "valid tag")
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v.strip_prefix('#') {
            Some(v) => Ok(TagType::Tag(v.to_string())),
            None => Ok(TagType::Item(v.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for TagType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TagVisitor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum RegistryEntryList {
    Single(TagType),
    Many(Vec<TagType>),
}

impl RegistryEntryList {
    #[allow(dead_code)]
    pub fn get_values(&self) -> Vec<TagType> {
        match self {
            RegistryEntryList::Single(s) => vec![s.clone()],
            RegistryEntryList::Many(s) => s.clone(),
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SlotTypeVisitor;
        impl<'de> Visitor<'de> for SlotTypeVisitor {
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "valid ingredient slot")
            }

            type Value = RegistryEntryList;

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(RegistryEntryList::Single(TagVisitor.visit_str(v)?))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
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

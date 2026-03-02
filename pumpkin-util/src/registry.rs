use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

/// Represents either a single item or a tag reference.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TagType {
    /// A single item identified by its name.
    Item(String),
    /// A tag reference, usually prefixed with `#`.
    Tag(String),
}

impl TagType {
    /// Serializes the tag type into a string representation.
    ///
    /// # Returns
    /// - For `Item`, returns the item name.
    /// - For `Tag`, returns the tag prefixed with `#`.
    #[must_use]
    pub fn serialize(&self) -> String {
        match self {
            Self::Item(name) => name.clone(),
            Self::Tag(tag) => format!("#{tag}"),
        }
    }
}

/// Visitor for deserializing a `TagType` from a string.
pub struct TagVisitor;

impl Visitor<'_> for TagVisitor {
    type Value = TagType;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "valid tag")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        v.strip_prefix('#').map_or_else(
            || Ok(TagType::Item(v.to_string())),
            |v| Ok(TagType::Tag(v.to_string())),
        )
    }
}

impl<'de> Deserialize<'de> for TagType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(TagVisitor)
    }
}

/// Represents either a single tag type or a list of multiple tag types.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum RegistryEntryList {
    /// A single tag type.
    Single(TagType),
    /// Multiple tag types.
    Many(Vec<TagType>),
}

impl RegistryEntryList {
    /// Converts the registry entry list into a flat vector of `TagType`.
    ///
    /// # Returns
    /// A vector containing all contained `TagType` values.
    #[must_use]
    pub fn into_vec(self) -> Vec<TagType> {
        match self {
            Self::Single(s) => vec![s],
            Self::Many(s) => s,
        }
    }
}

impl PartialEq<TagType> for RegistryEntryList {
    fn eq(&self, other: &TagType) -> bool {
        match self {
            Self::Single(ingredient) => other == ingredient,
            Self::Many(ingredients) => ingredients.contains(other),
        }
    }
}

impl<'de> Deserialize<'de> for RegistryEntryList {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SlotTypeVisitor;
        impl<'de> Visitor<'de> for SlotTypeVisitor {
            type Value = RegistryEntryList;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "valid ingredient slot")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(RegistryEntryList::Single(TagVisitor.visit_str(v)?))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut ingredients: Vec<TagType> = vec![];
                while let Some(element) = seq.next_element()? {
                    ingredients.push(element);
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

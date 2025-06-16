use std::{
    io::{Read, Write},
    num::NonZeroUsize,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};

use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    pub namespace: String,
    pub path: String,
}

impl ResourceLocation {
    pub fn vanilla(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
    pub fn pumpkin(path: &str) -> Self {
        Self {
            namespace: "pumpkin".to_string(),
            path: path.to_string(),
        }
    }
    pub fn get(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}
impl ResourceLocation {
    /// The maximum number of bytes for a [`ResourceLocation`] is the same as for a normal [`String`].
    const MAX_SIZE: NonZeroUsize = NonZeroUsize::new(i16::MAX as usize).unwrap();

    pub fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        write.write_string_bounded(&self.to_string(), Self::MAX_SIZE.get())
    }

    pub fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let resource_location = read.get_string_bounded(Self::MAX_SIZE.get())?;
        match resource_location.split_once(":") {
            Some((namespace, path)) => Ok(ResourceLocation {
                namespace: namespace.to_string(),
                path: path.to_string(),
            }),
            None => Err(ReadingError::Incomplete("ResourceLocation".to_string())),
        }
    }
}

impl Serialize for ResourceLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ResourceLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ResourceLocationVisitor;

        impl Visitor<'_> for ResourceLocationVisitor {
            type Value = ResourceLocation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid resource location (namespace:path)")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&v)
            }

            fn visit_str<E>(self, resource_location: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match resource_location.split_once(":") {
                    Some((namespace, path)) => Ok(ResourceLocation {
                        namespace: namespace.to_string(),
                        path: path.to_string(),
                    }),
                    None => Err(serde::de::Error::custom("resource location can't be split")),
                }
            }
        }
        deserializer.deserialize_str(ResourceLocationVisitor)
    }
}

impl std::fmt::Display for ResourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

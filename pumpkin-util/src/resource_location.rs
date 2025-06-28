use std::num::NonZeroUsize;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    pub namespace: String,
    pub path: String,
}

impl ResourceLocation {
    /// The maximum number of bytes for a [`ResourceLocation`] is the same as for a normal [`String`].
    pub const MAX_SIZE: NonZeroUsize = NonZeroUsize::new(i16::MAX as usize).unwrap();

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
}

impl std::fmt::Display for ResourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
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

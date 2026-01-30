use std::{num::NonZeroUsize, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    pub namespace: String,
    pub path: String,
}

impl ResourceLocation {
    /// The maximum number of bytes for a [`ResourceLocation`] is the same as for a normal [`String`].
    pub const MAX_SIZE: NonZeroUsize = NonZeroUsize::new(i16::MAX as usize).unwrap();

    #[must_use]
    pub fn from(location: &str) -> Self {
        let names = location.split_once(':').unwrap_or_default();
        Self {
            namespace: names.0.to_string(),
            path: names.1.to_string(),
        }
    }

    #[must_use]
    pub fn vanilla(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }

    #[must_use]
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

impl FromStr for ResourceLocation {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(':') {
            Some((namespace, path)) => Ok(Self {
                namespace: namespace.to_string(),
                path: path.to_string(),
            }),
            None => Err(()),
        }
    }
}

impl Serialize for ResourceLocation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ResourceLocation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ResourceLocationVisitor;

        impl Visitor<'_> for ResourceLocationVisitor {
            type Value = ResourceLocation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid resource location (namespace:path)")
            }

            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }

            fn visit_str<E: serde::de::Error>(
                self,
                resource_location: &str,
            ) -> Result<Self::Value, E> {
                match resource_location.split_once(':') {
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

pub trait ToResourceLocation: Sized {
    fn to_resource_location(&self) -> ResourceLocation;
}

pub trait FromResourceLocation: Sized {
    fn from_resource_location(resource_location: &ResourceLocation) -> Option<Self>;
}

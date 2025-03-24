use std::{
    borrow::Cow,
    io::{Read, Write},
    num::NonZeroUsize,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};

use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};

use super::Codec;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier<'a> {
    pub namespace: Cow<'a, str>,
    pub path: Cow<'a, str>,
}

impl<'a> Identifier<'a> {
    pub fn vanilla<P: Into<Cow<'a, str>>>(path: P) -> Self {
        Self {
            namespace: Cow::Borrowed("minecraft"),
            path: path.into(),
        }
    }
}

impl<'a> Codec<Identifier<'a>> for Identifier<'a> {
    const MAX_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(i16::MAX as usize) };

    fn written_size(&self) -> usize {
        self.namespace.len() + self.path.len()
    }

    fn encode(&self, writer: &mut impl Write) -> Result<(), WritingError> {
        let combined = format!("{}:{}", self.namespace, self.path);
        writer.write_string_bounded(&combined, Self::MAX_SIZE.get())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, ReadingError> {
        let identifier = reader.get_string_bounded(Self::MAX_SIZE.get())?;
        let (namespace, path) = identifier
            .split_once(':')
            .ok_or(ReadingError::Incomplete("Identifier".to_string()))?;

        Ok(Self {
            namespace: Cow::Owned(namespace.to_owned()),
            path: Cow::Owned(path.to_owned()),
        })
    }
}

impl Serialize for Identifier<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct IdentifierVisitor;

impl<'de> Visitor<'de> for IdentifierVisitor {
    type Value = Identifier<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid identifier (namespace:path)")
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let (namespace, path) = value.split_once(':').ok_or_else(|| {
            serde::de::Error::custom("Identifier must contain namespace and path separated by ':'")
        })?;
        Ok(Identifier {
            namespace: Cow::Borrowed(namespace),
            path: Cow::Borrowed(path),
        })
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(value.to_owned())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let (namespace, path) = value.split_once(':').ok_or_else(|| {
            serde::de::Error::custom("Identifier must contain namespace and path separated by ':'")
        })?;
        Ok(Identifier {
            namespace: Cow::Owned(namespace.to_owned()),
            path: Cow::Owned(path.to_owned()),
        })
    }
}

impl<'de> Deserialize<'de> for Identifier<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(IdentifierVisitor)
    }
}

impl std::fmt::Display for Identifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

use std::{fmt, str::FromStr};

/// A Minecraft resource identifier: `namespace:path`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    #[must_use]
    pub fn new(namespace: &str, path: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            path: path.to_string(),
        }
    }

    /// Parse from a string like `"minecraft:stone"` or `"stone"` (defaults to `minecraft:`).
    #[must_use]
    pub fn parse(id: &str) -> Self {
        if let Some((ns, p)) = id.split_once(':') {
            Self::new(ns, p)
        } else {
            Self::new("minecraft", id)
        }
    }

    #[must_use]
    pub fn with_prefix(self, prefix: &str) -> Self {
        Self {
            namespace: self.namespace,
            path: format!("{prefix}/{}", self.path),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

impl FromStr for Identifier {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

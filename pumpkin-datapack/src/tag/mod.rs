pub mod loader;
pub mod registry;

use crate::Identifier;
use serde::Deserialize;

/// A single entry in a tag file.
#[derive(Debug, Clone)]
pub enum TagEntry {
    /// A direct element reference (e.g., `"minecraft:stone"`).
    Element(Identifier, bool),
    /// A reference to another tag (e.g., `"#minecraft:base_stone_overworld"`).
    TagReference(Identifier, bool),
}

/// Deserialized tag JSON file.
#[derive(Debug, Clone, Deserialize)]
pub struct TagFile {
    #[serde(default)]
    pub replace: bool,
    #[serde(default)]
    pub values: Vec<serde_json::Value>,
}

/// A resolved tag: list of element IDs.
#[derive(Debug, Clone)]
pub struct Tag {
    pub entries: Vec<Identifier>,
}

impl Tag {
    #[must_use]
    pub const fn new(entries: Vec<Identifier>) -> Self {
        Self { entries }
    }
}

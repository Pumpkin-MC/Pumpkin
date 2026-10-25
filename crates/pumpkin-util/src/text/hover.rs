use std::{borrow::Cow, vec};

use serde::{Deserialize, Serialize};

use super::{TextComponent, TextComponentBase};

/// An item's data component patch in a `show_item` hover event, kept as the
/// JSON it was given (`{"minecraft:enchantments":{"minecraft:sharpness":5}}`).
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct ItemComponents(pub serde_json::Value);

impl<'de> Deserialize<'de> for ItemComponents {
    /// Only a JSON object is a component patch.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        if value.is_object() {
            Ok(Self(value))
        } else {
            Err(serde::de::Error::custom(
                "show_item components must be an object",
            ))
        }
    }
}

impl std::hash::Hash for ItemComponents {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_string().hash(state);
    }
}

impl ItemComponents {
    /// The components as an NBT compound, as the network writes them.
    #[must_use]
    pub fn to_nbt(&self) -> pumpkin_nbt::tag::NbtTag {
        json_to_nbt(&self.0)
    }
}

/// JSON as NBT: whole numbers as the smallest integer tag that holds them.
fn json_to_nbt(value: &serde_json::Value) -> pumpkin_nbt::tag::NbtTag {
    use pumpkin_nbt::tag::NbtTag;
    match value {
        serde_json::Value::Null => NbtTag::Compound(pumpkin_nbt::compound::NbtCompound::new()),
        serde_json::Value::Bool(b) => NbtTag::Byte(i8::from(*b)),
        serde_json::Value::Number(n) => n.as_i64().map_or_else(
            || NbtTag::Double(n.as_f64().unwrap_or(0.0)),
            |i| {
                i8::try_from(i).map_or_else(
                    |_| {
                        i16::try_from(i).map_or_else(
                            |_| i32::try_from(i).map_or(NbtTag::Long(i), NbtTag::Int),
                            NbtTag::Short,
                        )
                    },
                    NbtTag::Byte,
                )
            },
        ),
        serde_json::Value::String(s) => NbtTag::String(s.clone().into_boxed_str()),
        serde_json::Value::Array(list) => NbtTag::List(list.iter().map(json_to_nbt).collect()),
        serde_json::Value::Object(map) => {
            let mut compound = pumpkin_nbt::compound::NbtCompound::new();
            for (key, value) in map {
                compound.put(key, json_to_nbt(value));
            }
            NbtTag::Compound(compound)
        }
    }
}

/// Represents the hover event action in a chat component.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum HoverEvent {
    /// Displays a tooltip with the given text.
    ShowText { value: Vec<TextComponentBase> },
    /// Shows an item.
    ShowItem {
        /// Resource identifier of the item.
        id: Cow<'static, str>,
        /// Number of the items in the stack.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<i32>,
        /// The stack's data components (`minecraft:enchantments`, ...), as
        /// given.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        components: Option<ItemComponents>,
    },
    /// Shows an entity.
    ShowEntity {
        /// The entity's ID Entity Type.
        id: Cow<'static, str>,
        /// The entity's UUID
        /// The UUID cannot use `uuid::Uuid` because its serialization parses it into bytes, so its double bytes serialized.
        uuid: Cow<'static, str>,
        /// Optional custom name for the entity.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<Vec<TextComponentBase>>,
    },
}

impl HoverEvent {
    /// Creates a new hover event that displays text.
    ///
    /// # Arguments
    /// - `text` – The text component to display in the tooltip.
    ///
    /// # Returns
    /// A `HoverEvent::ShowText` variant containing the provided text.
    #[must_use]
    pub fn show_text(text: TextComponent) -> Self {
        Self::ShowText {
            value: vec![text.0],
        }
    }

    /// Creates a new hover event that displays entity information.
    ///
    /// # Arguments
    /// - `uuid` – The entity's UUID as a string.
    /// - `kind` – The entity type identifier (e.g., "minecraft:pig").
    /// - `name` – Optional custom name for the entity.
    ///
    /// # Returns
    /// A `HoverEvent::ShowEntity` variant containing the entity information.
    pub fn show_entity<P: Into<Cow<'static, str>>>(
        uuid: P,
        kind: P,
        name: Option<TextComponent>,
    ) -> Self {
        Self::ShowEntity {
            id: kind.into(),
            uuid: uuid.into(),
            name: match name {
                Some(name) => Some(vec![name.0]),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_components_must_be_an_object() {
        assert!(serde_json::from_str::<ItemComponents>(r#"{"minecraft:damage":3}"#).is_ok());
        for bad in ["[]", r#""x""#, "3", "null", "true"] {
            assert!(
                serde_json::from_str::<ItemComponents>(bad).is_err(),
                "{bad}"
            );
        }
    }
}

use std::{borrow::Cow, vec};

use serde::{Deserialize, Serialize};

use super::{TextComponent, TextComponentBase};

/// Represents the hover event action in a chat component.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "action", rename_all = "snake_case", try_from = "RawHoverEvent")]
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
        // #[serde(default, skip_serializing_if = "Option::is_none")]
        // components: Option<Cow<'static, str>>,
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

// Mirrors vanilla's `TextComponentHoverAndClickEventFix`: before 1.21.5 the
// event fields were nested under `contents`, and `show_entity` used `type`
// for the entity type and `id` for the UUID.
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
#[expect(
    clippy::enum_variant_names,
    reason = "variant names are the vanilla action names"
)]
enum RawHoverEvent {
    ShowText {
        #[serde(alias = "contents")]
        value: RawComponents,
    },
    ShowItem {
        #[serde(default)]
        id: Option<Cow<'static, str>>,
        #[serde(default)]
        count: Option<i32>,
        #[serde(default)]
        contents: Option<LegacyItem>,
    },
    ShowEntity {
        #[serde(default)]
        id: Option<Cow<'static, str>>,
        #[serde(default)]
        uuid: Option<Cow<'static, str>>,
        #[serde(default)]
        name: Option<RawComponents>,
        #[serde(default)]
        contents: Option<LegacyEntity>,
    },
}

#[derive(Deserialize)]
struct LegacyItem {
    id: Cow<'static, str>,
    #[serde(default)]
    count: Option<i32>,
}

#[derive(Deserialize)]
struct LegacyEntity {
    #[serde(rename = "type")]
    kind: Cow<'static, str>,
    id: Cow<'static, str>,
    #[serde(default)]
    name: Option<RawComponents>,
}

// Keeps an array as separate entries, so a save and reload doesn't nest it under a new parent.
#[derive(Deserialize)]
#[serde(untagged)]
enum RawComponents {
    Many(Vec<TextComponent>),
    One(TextComponent),
}

impl From<RawComponents> for Vec<TextComponentBase> {
    fn from(raw: RawComponents) -> Self {
        match raw {
            RawComponents::Many(components) => components.into_iter().map(|c| c.0).collect(),
            RawComponents::One(component) => vec![component.0],
        }
    }
}

impl TryFrom<RawHoverEvent> for HoverEvent {
    type Error = &'static str;

    fn try_from(raw: RawHoverEvent) -> Result<Self, Self::Error> {
        Ok(match raw {
            RawHoverEvent::ShowText { value } => Self::ShowText {
                value: value.into(),
            },
            RawHoverEvent::ShowItem {
                id,
                count,
                contents,
            } => match contents {
                Some(item) => Self::ShowItem {
                    id: item.id,
                    count: item.count,
                },
                None => Self::ShowItem {
                    id: id.ok_or("show_item hover event is missing `id`")?,
                    count,
                },
            },
            RawHoverEvent::ShowEntity {
                id,
                uuid,
                name,
                contents,
            } => match contents {
                Some(entity) => Self::ShowEntity {
                    id: entity.kind,
                    uuid: entity.id,
                    name: entity.name.map(Into::into),
                },
                None => Self::ShowEntity {
                    id: id.ok_or("show_entity hover event is missing `id`")?,
                    uuid: uuid.ok_or("show_entity hover event is missing `uuid`")?,
                    name: name.map(Into::into),
                },
            },
        })
    }
}

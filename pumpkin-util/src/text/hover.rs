use std::{borrow::Cow, vec};

use serde::{Deserialize, Serialize};

use super::{TextComponent, TextComponentBase};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "action", content = "contents", rename_all = "snake_case")]
pub enum HoverEvent {
    /// Displays a tooltip with the given text.
    ShowText(Vec<TextComponentBase>),
    /// Shows an item.
    ShowItem {
        /// Resource identifier of the item
        id: Cow<'static, str>,
        /// Number of the items in the stack
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<i32>,
        /// NBT information about the item (sNBT format)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<Cow<'static, str>>,
    },
    /// Shows an entity.
    ShowEntity {
        /// The entity's UUID
        /// The UUID cannot use uuid::Uuid because its serialization parses it into bytes, so its double bytes serialized
        id: Cow<'static, str>,
        /// Resource identifier of the entity
        #[serde(default, skip_serializing_if = "Option::is_none")]
        r#type: Option<Cow<'static, str>>,
        /// Optional custom name for the entity
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<Vec<TextComponentBase>>,
    },
}

impl HoverEvent {
    pub fn show_text(text: TextComponent) -> Self {
        Self::ShowText(vec![text.0])
    }
    pub fn show_entity<P>(id: P, kind: P, name: Option<TextComponent>) -> Self
    where
        P: Into<Cow<'static, str>>,
    {
        Self::ShowEntity {
            id: id.into(),
            r#type: Some(kind.into()),
            name: match name {
                Some(name) => Some(vec![name.0]),
                None => None,
            },
        }
    }
}

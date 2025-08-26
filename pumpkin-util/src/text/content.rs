use crate::text::TextComponentBase;
use core::str;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum TextContent {
    /// Raw text
    Text { text: Cow<'static, str> },
    /// Translated text
    Translate {
        translate: Cow<'static, str>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fallback: Option<Cow<'static, str>>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        with: Vec<TextComponentBase>,
    },
    /// Displays a score from the scoreboard.
    /// #### Requires component resolution.
    #[serde(skip_serializing)]
    Scoreboard { score: ScoreboardValue },
    /// Displays the name of one or more entities found by a selector.
    /// #### Requires component resolution.
    #[serde(skip_serializing)]
    EntityNames {
        selector: Cow<'static, str>,
        separator: Option<Vec<TextComponentBase>>,
        /// Sender Uuid
        sender: Option<Uuid>,
    },
    /// A keybind identifier
    /// https://minecraft.wiki/w/Controls#Configurable_controls
    Keybind { keybind: Cow<'static, str> },
    /// Displays NBT values from entities, block entities, or command storage.
    /// #### Requires component resolution.
    #[serde(skip_serializing)]
    Nbt {
        #[serde(flatten)]
        value: NbtValue,
    },
    /// Displays a single sprite from texture atlas as a character. Sprites are rendered as 8x8 pixels squares.
    Sprite {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        atlas: Option<Cow<'static, str>>,
        sprite: Cow<'static, str>,
    },
    /// A custom translated text
    /// #### Requires component resolution.
    #[serde(skip_serializing)]
    Custom {
        key: Cow<'static, str>,
        with: Vec<TextComponentBase>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ScoreboardValue {
    pub name: Cow<'static, str>,
    pub objective: Cow<'static, str>,
    /// Sender Uuid
    pub sender: Option<Uuid>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NbtValue {
    pub source: Option<NbtSource>,
    pub nbt: Cow<'static, str>,
    pub interpret: Option<bool>,
    pub separator: Option<Vec<TextComponentBase>>,
    pub block: Option<Cow<'static, str>>,
    pub entity: Option<Cow<'static, str>>,
    pub storage: Option<Cow<'static, str>>,
    /// Sender Uuid
    pub sender: Option<Uuid>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum NbtSource {
    Block,
    Entity,
    Storage,
}

impl Default for TextContent {
    fn default() -> Self {
        TextContent::Text {
            text: Cow::from(""),
        }
    }
}

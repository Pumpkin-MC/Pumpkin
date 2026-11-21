use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// A JSON payload kept verbatim for `show_dialog` and custom click actions.
///
/// The value is boxed to keep [`super::style::Style`] small.
#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ClickPayload(pub Box<serde_json::Value>);

impl<'de> Deserialize<'de> for ClickPayload {
    /// NBT has no null payload, so a null cannot round-trip.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        if value.is_null() {
            return Err(serde::de::Error::custom("click payload cannot be null"));
        }
        Ok(Self(Box::new(value)))
    }
}

impl ClickPayload {
    /// The payload as NBT, as the network writes it.
    #[must_use]
    pub fn to_nbt(&self) -> pumpkin_nbt::tag::NbtTag {
        super::hover::json_to_nbt(&self.0)
    }
}

/// Action to take on click of the text.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Eq, Hash)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClickEvent {
    /// Opens a URL.
    OpenUrl { url: Cow<'static, str> },
    /// Opens a file.
    OpenFile { path: Cow<'static, str> },
    /// Works in signs but only on the root text component.
    RunCommand { command: Cow<'static, str> },
    /// Shows a dialog.
    ShowDialog { dialog: ClickPayload },
    /// Runs a custom click action.
    Custom {
        id: Cow<'static, str>,
        payload: Option<ClickPayload>,
    },
    /// Replaces the contents of the chat box with the text, not necessarily a
    /// command.
    SuggestCommand { command: Cow<'static, str> },
    /// Only usable within written books. Changes the page of the book. Indexing
    /// starts at 1.
    ChangePage { page: u32 },
    /// Copies the given text to the system clipboard.
    CopyToClipboard { value: Cow<'static, str> },
}

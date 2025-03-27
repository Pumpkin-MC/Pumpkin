use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Action to take on click of the text.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Eq, Hash)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClickEvent {
    /// Opens a URL.
    OpenUrl { url: Cow<'static, str> },
    /// Opens a file.
    OpenFile { path: Cow<'static, str> },
    /// Works in signs, but only on the root text component.
    RunCommand { command: Cow<'static, str> },
    /// Replaces the contents of the chat box with the text, not necessarily a
    /// command.
    SuggestCommand { command: Cow<'static, str> },
    /// Only usable within written books. Changes the page of the book. Indexing
    /// starts at 1.
    ChangePage { page: u32 },
    /// Copies the given text to system clipboard.
    CopyToClipboard { value: Cow<'static, str> },
}

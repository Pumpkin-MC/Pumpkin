use std::borrow::Cow;

use pumpkin_util::text::style::Style;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatType<'a> {
    chat: Decoration<'a>,
    narration: Decoration<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decoration<'a> {
    translation_key: Cow<'a, str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    style: Option<Style>,
    parameters: Vec<Cow<'a, str>>,
}

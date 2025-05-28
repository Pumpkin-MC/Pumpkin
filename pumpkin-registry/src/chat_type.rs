use pumpkin_util::text::style::Style;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatType {
    pub chat: Decoration,
    pub narration: Decoration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decoration {
    pub translation_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
    pub parameters: Vec<String>,
}

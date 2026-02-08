use pumpkin_data::packet::serverbound::PLAY_EDIT_BOOK;
use pumpkin_macros::java_packet;

use crate::VarInt;

/// Sent when the player edits or signs a book.
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_EDIT_BOOK)]
pub struct SEditBook {
    pub slot: VarInt,
    pub pages: Vec<String>,
    pub title: Option<String>,
}

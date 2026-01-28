use pumpkin_data::packet::clientbound::PLAY_OPEN_SCREEN;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Instructs the client to open a specific type of GUI (inventory, chest, etc.).
///
/// This packet is sent when a player interacts with a block (like a chest)
/// or when a command/plugin forces an interface to open. It establishes a
/// `sync_id` which must be used in all subsequent "Set Slot" or "Click Slot"
/// packets to ensure the server and client are talking about the same window.
#[derive(Serialize)]
#[java_packet(PLAY_OPEN_SCREEN)]
pub struct COpenScreen<'a> {
    /// A unique identifier for the current window session.
    /// Typically increments by 1 for every new window opened.
    pub sync_id: VarInt,
    /// The ID of the window type to open (e.g., Generic 9x3, Crafting Table).
    /// See the table below for standard IDs.
    pub window_type: VarInt,
    /// The title displayed at the top of the GUI.
    /// Supports full JSON formatting (colors, bold, etc.).
    pub window_title: &'a TextComponent,
}

impl<'a> COpenScreen<'a> {
    #[must_use]
    pub fn new(window_id: VarInt, window_type: VarInt, window_title: &'a TextComponent) -> Self {
        Self {
            sync_id: window_id,
            window_type,
            window_title,
        }
    }
}

use pumpkin_data::packet::serverbound::CONFIG_CLIENT_INFORMATION;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Sent by the client to inform the server about its local settings
#[derive(serde::Deserialize, Serialize)]
#[java_packet(CONFIG_CLIENT_INFORMATION)]
pub struct SClientInformationConfig {
    /// The language code used by the client (e.g., "`en_us`")
    pub locale: String,
    /// The maximum number of chunks the client renders
    pub view_distance: i8,
    /// Visibility of chat messages (0: Enabled, 1: Commands Only, 2: Hidden)
    pub chat_mode: VarInt,
    /// Whether the client wants chat colors/formatting rendered
    pub chat_colors: bool,
    /// Bitmask representing displayed skin parts (e.g., cape, jacket, sleeves)
    pub skin_parts: u8,
    /// The player's dominant hand (0: Left, 1: Right)
    pub main_hand: VarInt,
    /// Whether the client wants text filtering (e.g., for profanity) enabled
    pub text_filtering: bool,
    /// Whether the player should appear in the server's online player list
    pub server_listing: bool,
}

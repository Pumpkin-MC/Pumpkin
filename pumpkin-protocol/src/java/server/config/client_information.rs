use pumpkin_data::packet::serverbound::CONFIG_CLIENT_INFORMATION;
use pumpkin_macros::java_packet;

use crate::VarInt;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

/// Sent by the client to inform the server about its local settings
#[java_packet(CONFIG_CLIENT_INFORMATION)]
pub struct SClientInformationConfig<'a> {
    /// The language code used by the client (e.g., "`en_us`")
    pub locale: &'a str,
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

impl<'a> ServerPacket<'a> for SClientInformationConfig<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            locale: bytebuf.get_str_borrowed()?,
            view_distance: bytebuf.get_i8()?,
            chat_mode: bytebuf.get_var_int()?,
            chat_colors: bytebuf.get_bool()?,
            skin_parts: bytebuf.get_u8()?,
            main_hand: bytebuf.get_var_int()?,
            text_filtering: bytebuf.get_bool()?,
            server_listing: bytebuf.get_bool()?,
        })
    }
}

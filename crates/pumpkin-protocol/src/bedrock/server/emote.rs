// Last verified for v2169

use crate::codec::var_uint::VarUInt;
use crate::codec::var_ulong::VarULong;
use crate::serial::{PacketRead, PacketReadSlice, PacketWrite};
use pumpkin_macros::packet;
use std::borrow::Cow;

/// Flag indicating the emote was triggered server-side.
pub const EMOTE_FLAG_SERVER_SIDE: u8 = 1 << 0;
/// Flag indicating chat notifications for this emote should be suppressed.
pub const EMOTE_FLAG_MUTE_CHAT: u8 = 1 << 1;

/// Transmits player emote animations to the server and broadcasts them to nearby players.
#[derive(Debug, PacketRead, PacketReadSlice, PacketWrite)]
#[packet(138)]
pub struct SEmote<'a> {
    /// Runtime entity ID of the emoting player.
    pub actor_runtime_id: VarULong,
    /// UUID identifier string of the emote piece being performed.
    pub emote_id: Cow<'a, str>,
    /// Duration of the emote animation in ticks.
    pub emote_length_ticks: VarUInt,
    /// Xbox Live User ID of the player.
    pub xuid: Cow<'a, str>,
    /// Platform ID string of the player.
    pub platform_id: Cow<'a, str>,
    /// Bitflags modifying emote playback behavior (e.g. mute chat).
    pub flags: u8,
}

// Last verified for v2169

use crate::codec::var_uint::VarUInt;
use crate::codec::var_ulong::VarULong;
use crate::serial::{PacketRead, PacketReadSlice, PacketWrite};
use pumpkin_macros::packet;
use std::borrow::Cow;

pub const EMOTE_FLAG_SERVER_SIDE: u8 = 1 << 0;
pub const EMOTE_FLAG_MUTE_CHAT: u8 = 1 << 1;

#[derive(Debug, PacketRead, PacketReadSlice, PacketWrite)]
#[packet(138)]
pub struct SEmote<'a> {
    pub actor_runtime_id: VarULong,
    pub emote_id: Cow<'a, str>,
    pub emote_length_ticks: VarUInt,
    pub xuid: Cow<'a, str>,
    pub platform_id: Cow<'a, str>,
    pub flags: u8,
}

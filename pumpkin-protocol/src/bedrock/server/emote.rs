use crate::codec::var_ulong::VarULong;
use crate::serial::{PacketRead, PacketWrite};
use pumpkin_macros::packet;

pub const EMOTE_FLAG_SERVER_SIDE: u8 = 1 << 0;
pub const EMOTE_FLAG_MUTE_CHAT: u8 = 1 << 1;

#[derive(PacketRead, PacketWrite, Debug)]
#[packet(138)]
pub struct SEmote {
    pub runtime_entity_id: VarULong,
    pub emote_id: String,
    pub emote_length: u32,
    pub xuid: String,
    pub platform_id: String,
    pub flags: u8,
}

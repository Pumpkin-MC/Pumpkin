// Last verified for v2169

use crate::{codec::var_ulong::VarULong, serial::PacketRead};
use pumpkin_macros::packet;

#[derive(PacketRead)]
#[packet(113)]
pub struct SSetLocalPlayerAsInitialized {
    pub player_id: VarULong,
}

use crate::{codec::var_ulong::VarULong, serial::PacketRead};
use pumpkin_macros::packet;

#[derive(PacketRead)]
#[packet(113)]
pub struct SSetLocalPlayerAsInitialized {
    pub runtime_entity_id: VarULong,
}

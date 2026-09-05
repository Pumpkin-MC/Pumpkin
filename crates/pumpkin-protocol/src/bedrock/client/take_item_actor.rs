// Last verified for v2169

use crate::{codec::var_ulong::VarULong, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(17)]
pub struct CTakeItemActor {
    pub item_runtime_id: VarULong,
    pub actor_runtime_id: VarULong,
}

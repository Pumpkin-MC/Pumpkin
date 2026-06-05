use crate::{codec::var_ulong::VarULong, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(17)]
pub struct CTakeItemActor {
    // https://github.com/Mojang/bedrock-protocol-docs/blob/main/docs/TakeItemActorPacket.html
    pub item_runtime_id: VarULong,
    pub actor_runtime_id: VarULong,
}
impl CTakeItemActor {
    #[must_use]
    pub const fn new(item_runtime_id: VarULong, actor_runtime_id: VarULong) -> Self {
        Self {
            item_runtime_id,
            actor_runtime_id,
        }
    }
}

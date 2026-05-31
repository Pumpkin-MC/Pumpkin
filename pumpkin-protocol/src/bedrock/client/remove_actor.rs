use crate::{codec::var_long::VarLong, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(14)]
pub struct CRemoveActor {
    pub entity_unique_id: VarLong,
}

impl CRemoveActor {
    #[must_use]
    pub const fn new(entity_unique_id: VarLong) -> Self {
        Self { entity_unique_id }
    }
}

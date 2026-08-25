// Last verified for v2169

use crate::{codec::var_long::VarLong, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(14)]
pub struct CRemoveActor {
    pub target_actor_id: VarLong,
}

impl CRemoveActor {
    #[must_use]
    pub const fn new(target_actor_id: VarLong) -> Self {
        Self { target_actor_id }
    }
}

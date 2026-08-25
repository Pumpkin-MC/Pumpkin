// Last verified for v2169

use crate::serial::PacketWrite;
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(106)]
pub struct CRemoveObjective {
    pub objective_name: String,
}

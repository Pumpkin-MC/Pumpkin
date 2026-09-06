// Last verified for v2169

use crate::{codec::var_int::VarInt, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(107)]
pub struct CSetDisplayObjective {
    pub display_slot_name: String,
    pub objective_name: String,
    pub objective_display_name: String,
    pub criteria_name: String,
    pub sort_order: VarInt,
}

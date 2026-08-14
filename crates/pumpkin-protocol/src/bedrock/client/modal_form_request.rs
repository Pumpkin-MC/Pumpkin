use crate::{codec::var_uint::VarUInt, serial::PacketWrite};
use pumpkin_macros::packet;

#[packet(100)]
#[derive(PacketWrite)]
pub struct CModalFormRequest {
    pub form_id: VarUInt,
    pub form_data: String,
}

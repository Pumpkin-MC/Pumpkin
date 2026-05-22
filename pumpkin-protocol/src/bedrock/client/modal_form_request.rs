use crate::{codec::var_int::VarInt, serial::PacketWrite};
use pumpkin_macros::packet;

#[packet(100)]
#[derive(PacketWrite)]
pub struct CModalFormRequest {
    pub form_id: VarInt,
    pub form_data: String,
}

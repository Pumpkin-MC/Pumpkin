use crate::codec::var_uint::VarUInt;
use crate::serial::{PacketRead, PacketReadSlice};
use pumpkin_macros::packet;
use std::borrow::Cow;

#[derive(Debug, PacketRead, PacketReadSlice)]
#[packet(101)]
pub struct SModalFormResponse<'a> {
    pub form_id: VarUInt,
    pub form_data: Option<Cow<'a, str>>,
    pub cancel_reason: Option<u8>,
}

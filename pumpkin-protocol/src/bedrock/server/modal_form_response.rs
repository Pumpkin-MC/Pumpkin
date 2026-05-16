use crate::codec::var_int::VarInt;
use crate::serial::PacketRead;
use pumpkin_macros::packet;
use std::io::{Error, Read};

#[packet(101)]
pub struct SModalFormResponse {
    pub form_id: VarInt,
    pub form_data: Option<String>,
}

impl PacketRead for SModalFormResponse {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let form_id = VarInt::read(reader)?;
        let form_data = Option::<String>::read(reader)?;
        Ok(Self { form_id, form_data })
    }
}

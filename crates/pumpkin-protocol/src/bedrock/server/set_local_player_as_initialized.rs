use crate::{codec::var_ulong::VarULong, serial::PacketRead};
use pumpkin_macros::packet;
use std::io::{Error, Read};

#[packet(113)]
pub struct SSetLocalPlayerAsInitialized {
    pub runtime_entity_id: VarULong,
}

impl PacketRead for SSetLocalPlayerAsInitialized {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            runtime_entity_id: VarULong::read(reader)?,
        })
    }
}

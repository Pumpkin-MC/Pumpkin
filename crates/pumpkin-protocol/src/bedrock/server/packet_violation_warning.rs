use std::io::{Error, Read};

use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketRead};

#[derive(Debug)]
#[packet(156)]
pub struct SPacketViolationWarning {
    pub violation_type: VarInt,
    pub severity: VarInt,
    pub packet_id: VarInt,
    pub context: String,
}

impl PacketRead for SPacketViolationWarning {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            violation_type: VarInt::read(reader)?,
            severity: VarInt::read(reader)?,
            packet_id: VarInt::read(reader)?,
            context: String::read(reader)?,
        })
    }
}

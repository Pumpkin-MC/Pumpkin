use crate::{
    bedrock::network_item::NetworkItemDescriptor,
    codec::{var_int::VarInt, var_ulong::VarULong},
    serial::PacketRead,
};
use pumpkin_macros::packet;
use std::io::{Error, Read};

#[derive(Debug)]
#[packet(31)]
pub struct SMobEquipment {
    pub entity_runtime_id: VarULong,
    pub stack_id: VarInt,
    pub item: Option<NetworkItemDescriptor>,
    pub inventory_slot: u8,
    pub hotbar_slot: u8,
    pub window_id: u8,
}

impl PacketRead for SMobEquipment {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let entity_runtime_id = VarULong::read(reader)?;
        let stack_id = VarInt::read(reader)?;
        let item = if stack_id.0 == 0 {
            None
        } else {
            Some(NetworkItemDescriptor::read(reader)?)
        };
        let inventory_slot = u8::read(reader)?;
        let hotbar_slot = u8::read(reader)?;
        let window_id = u8::read(reader)?;

        Ok(Self {
            entity_runtime_id,
            stack_id,
            item,
            inventory_slot,
            hotbar_slot,
            window_id,
        })
    }
}

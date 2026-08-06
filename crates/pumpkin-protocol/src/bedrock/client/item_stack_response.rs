use std::io::{Error, Write};

use crate::{
    bedrock::network_item::FullContainerName,
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};
use pumpkin_macros::packet;

#[derive(PacketWrite, Debug, Clone)]
pub struct ItemStackResponseSlotInfo {
    pub slot: u8,
    pub hotbar_slot: u8,
    pub count: u8,
    pub item_stack_id: VarInt,
    pub custom_name: String,
    pub filtered_custom_name: String,
    pub durability_correction: VarInt,
}

#[derive(PacketWrite, Debug, Clone)]
pub struct ItemStackResponseContainerInfo {
    pub container_name: FullContainerName,
    pub slots: Vec<ItemStackResponseSlotInfo>,
}

#[derive(Debug, Clone)]
pub struct ItemStackResponse {
    pub result: u8, // 0 = SUCCESS, 1 = ERROR
    pub request_id: VarInt,
    pub container_infos: Vec<ItemStackResponseContainerInfo>,
}

impl PacketWrite for ItemStackResponse {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.result.write(writer)?;
        self.request_id.write(writer)?;
        if self.result == 0 {
            VarUInt(self.container_infos.len() as u32).write(writer)?;
            for info in &self.container_infos {
                info.write(writer)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[packet(148)]
pub struct CItemStackResponse {
    pub responses: Vec<ItemStackResponse>,
}

impl PacketWrite for CItemStackResponse {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(self.responses.len() as u32).write(writer)?;
        for response in &self.responses {
            response.write(writer)?;
        }
        Ok(())
    }
}

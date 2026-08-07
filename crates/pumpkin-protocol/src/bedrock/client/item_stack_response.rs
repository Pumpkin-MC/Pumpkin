use std::io::{Error, Write};

use crate::{
    bedrock::network_item::FullContainerName,
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};
use pumpkin_macros::packet;

#[derive(Debug, Clone)]
pub struct ItemStackResponseSlotInfo {
    pub slot: u8,
    pub hotbar_slot: u8,
    pub count: u8,
    pub item_stack_id: VarInt,
    pub custom_name: String,
    pub filtered_custom_name: String,
    pub durability_correction: VarInt,
}

impl PacketWrite for ItemStackResponseSlotInfo {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        if !(-32768..=32767).contains(&self.durability_correction.0) {
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "durability correction must fit in an i16",
            ));
        }
        self.slot.write(writer)?;
        self.hotbar_slot.write(writer)?;
        self.count.write(writer)?;
        true.write(writer)?;
        (self.item_stack_id.0 > 0).write(writer)?;
        if self.item_stack_id.0 > 0 {
            self.item_stack_id.write(writer)?;
        }
        self.custom_name.write(writer)?;
        self.filtered_custom_name.write(writer)?;
        self.durability_correction.write(writer)
    }
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
        true.write(writer)?;
        (!self.container_infos.is_empty()).write(writer)?;
        if !self.container_infos.is_empty() {
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
        if self.responses.len() > 4096 {
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "item stack response count exceeds 4096",
            ));
        }
        VarUInt(self.responses.len() as u32).write(writer)?;
        for response in &self.responses {
            response.write(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_out_of_range_durability_correction() {
        let slot = ItemStackResponseSlotInfo {
            slot: 0,
            hotbar_slot: 0,
            count: 1,
            item_stack_id: VarInt(1),
            custom_name: String::new(),
            filtered_custom_name: String::new(),
            durability_correction: VarInt(32768),
        };
        assert!(slot.write(&mut Vec::new()).is_err());
    }
}

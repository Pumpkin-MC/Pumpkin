use std::io::{Error, Write};

use crate::{
    bedrock::network_item::FullContainerName,
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};
use pumpkin_macros::packet;

#[derive(Debug, Clone)]
pub struct ItemStackResponseSlotInfo {
    pub requested_slot: u8,
    pub slot: u8,
    pub amount: u8,
    pub item_stack_net_id: VarInt,
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
        self.requested_slot.write(writer)?;
        self.slot.write(writer)?;
        self.amount.write(writer)?;
        true.write(writer)?;
        (self.item_stack_net_id.0 > 0).write(writer)?;
        if self.item_stack_net_id.0 > 0 {
            self.item_stack_net_id.write(writer)?;
        }
        self.custom_name.write(writer)?;
        self.filtered_custom_name.write(writer)?;
        self.durability_correction.write(writer)
    }
}

#[derive(PacketWrite, Debug, Clone)]
pub struct ItemStackResponseContainerInfo {
    pub full_container_name: FullContainerName,
    pub slots: Vec<ItemStackResponseSlotInfo>,
}

#[derive(Debug, Clone)]
pub struct ItemStackResponseInfo {
    // TODO: proper enum
    pub result: u8, // 0 = SUCCESS, 1 = ERROR
    pub client_request_id: VarInt,
    pub containers: Vec<ItemStackResponseContainerInfo>,
}

impl PacketWrite for ItemStackResponseInfo {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.result.write(writer)?;
        self.client_request_id.write(writer)?;
        true.write(writer)?;
        (!self.containers.is_empty()).write(writer)?;
        if !self.containers.is_empty() {
            VarUInt(self.containers.len() as u32).write(writer)?;
            for info in &self.containers {
                info.write(writer)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[packet(148)]
pub struct CItemStackResponse {
    pub responses: Vec<ItemStackResponseInfo>,
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
            requested_slot: 0,
            slot: 0,
            amount: 1,
            item_stack_net_id: VarInt(1),
            custom_name: String::new(),
            filtered_custom_name: String::new(),
            durability_correction: VarInt(32768),
        };
        assert!(slot.write(&mut Vec::new()).is_err());
    }
}

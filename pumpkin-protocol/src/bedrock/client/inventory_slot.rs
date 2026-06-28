use crate::{
    bedrock::network_item::{FullContainerName, NetworkItemStackDescriptor},
    codec::var_uint::VarUInt,
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use std::io::{Error, Write};

#[packet(50)]
pub struct CInventorySlot {
    pub window_id: VarUInt,
    pub inventory_slot: VarUInt,
    pub container_name: Option<FullContainerName>,
    pub storage: Option<NetworkItemStackDescriptor>,
    pub item: NetworkItemStackDescriptor,
}

impl PacketWrite for CInventorySlot {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.window_id.write(writer)?;
        self.inventory_slot.write(writer)?;

        self.container_name.is_some().write(writer)?;
        if let Some(container_name) = &self.container_name {
            container_name.write(writer)?;
        }

        self.storage.is_some().write(writer)?;
        if let Some(storage) = &self.storage {
            storage.write(writer)?;
        }

        self.item.write(writer)?;
        Ok(())
    }
}

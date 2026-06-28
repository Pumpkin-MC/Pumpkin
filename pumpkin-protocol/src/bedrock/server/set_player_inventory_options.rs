use crate::serial::PacketRead;
use pumpkin_macros::packet;
use std::io::{Error, Read};

#[packet(307)]
pub struct SSetPlayerInventoryOptions {
    pub left_inventory_tab: u8,
    pub right_inventory_tab: u8,
    pub filtering: bool,
    pub inventory_layout: u8,
    pub crafting_layout: u8,
}

impl PacketRead for SSetPlayerInventoryOptions {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            left_inventory_tab: u8::read(reader)?,
            right_inventory_tab: u8::read(reader)?,
            filtering: bool::read(reader)?,
            inventory_layout: u8::read(reader)?,
            crafting_layout: u8::read(reader)?,
        })
    }
}

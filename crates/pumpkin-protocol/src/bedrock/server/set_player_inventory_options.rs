use crate::{codec::var_int::VarInt, serial::PacketRead};
use pumpkin_macros::packet;
use std::io::{Error, Read};

#[packet(307)]
pub struct SSetPlayerInventoryOptions {
    pub left_inventory_tab: VarInt,
    pub right_inventory_tab: VarInt,
    pub filtering: bool,
    pub inventory_layout: VarInt,
    pub crafting_layout: VarInt,
}

impl PacketRead for SSetPlayerInventoryOptions {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            left_inventory_tab: VarInt::read(reader)?,
            right_inventory_tab: VarInt::read(reader)?,
            filtering: bool::read(reader)?,
            inventory_layout: VarInt::read(reader)?,
            crafting_layout: VarInt::read(reader)?,
        })
    }
}

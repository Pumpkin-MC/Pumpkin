use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::{
    bedrock::network_item::NetworkItemDescriptor, codec::var_uint::VarUInt, serial::PacketWrite,
};

#[packet(145)]
pub struct CCreativeContent<'a> {
    // https://mojang.github.io/bedrock-protocol-docs/html/CreativeContentPacket.html
    pub groups: &'a [Group],
    pub entries: &'a [Entry],
}

impl PacketWrite for CCreativeContent<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(self.groups.len() as _).write(writer)?;
        for group in self.groups {
            group.write(writer)?;
        }

        VarUInt(self.entries.len() as _).write(writer)?;
        for entry in self.entries {
            entry.write(writer)?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone)]
#[repr(i32)]
pub enum CreativeCategory {
    Construction = 1,
    Nature = 2,
    Equipment = 3,
    Items = 4,
    CommandOnly = 5,
    Undefined = 6,
}

impl PacketWrite for CreativeCategory {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as i32).write(writer)?;
        Ok(())
    }
}

#[derive(PacketWrite)]
pub struct Group {
    pub creative_category: CreativeCategory,
    pub name: String,
    pub icon_item: NetworkItemDescriptor,
}

#[derive(PacketWrite)]
pub struct Entry {
    pub id: VarUInt,
    pub item: NetworkItemDescriptor,
    pub group_index: VarUInt,
}

use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::{
    bedrock::network_item::NetworkItemDescriptor, codec::var_uint::VarUInt, serial::PacketWrite,
};

#[packet(145)]
pub struct CreativeContent<'a> {
    // https://mojang.github.io/bedrock-protocol-docs/html/CreativeContentPacket.html
    pub groups: &'a [Group],
    pub entries: &'a [Entry],
}

impl PacketWrite for CreativeContent<'_> {
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

#[repr(i32)]
#[allow(unused)]
enum CreativeCategory {
    Construction = 1,
    Nature = 2,
    Equipment = 3,
    Items = 4,
    CommandOnly = 5,
    Undefined = 6,
}

#[derive(PacketWrite)]
pub struct Group {
    pub creative_category: i32,
    pub name: String,
    pub icon_item: NetworkItemDescriptor,
}

#[derive(PacketWrite)]
pub struct Entry {
    pub id: VarUInt,
    pub item: NetworkItemDescriptor,
    pub group_index: VarUInt,
}

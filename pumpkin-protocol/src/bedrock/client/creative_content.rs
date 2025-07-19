use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::{
    BClientPacket, bedrock::network_item::NetworkItemDescriptor, codec::var_uint::VarUInt,
    serial::PacketWrite,
};

#[packet(145)]
pub struct CreativeContent<'a> {
    // https://mojang.github.io/bedrock-protocol-docs/html/CreativeContentPacket.html
    pub groups: &'a [Group],
    pub entries: &'a [Entry],
}

impl BClientPacket for CreativeContent<'_> {
    fn write_packet(&self, mut writer: impl Write) -> Result<(), Error> {
        VarUInt(self.groups.len() as _).write(&mut writer)?;
        for group in self.groups {
            group.write(&mut writer)?;
        }

        VarUInt(self.entries.len() as _).write(&mut writer)?;
        for entry in self.entries {
            entry.write(&mut writer)?;
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

use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::{
    BClientPacket,
    codec::{var_int::VarInt, var_uint::VarUInt},
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

#[derive(Default)]
pub struct NetworkItemDescriptor {
    // I hate mojang
    // https://mojang.github.io/bedrock-protocol-docs/html/NetworkItemInstanceDescriptor.html
    pub id: VarInt,
    pub stack_size: u32,
    pub aux_value: VarUInt,
    pub block_runtime_id: VarInt,
    pub user_data_buffer: ItemInstanceUserData,
}

impl PacketWrite for NetworkItemDescriptor {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.id.write(writer)?;
        if self.id.0 != 0 {
            self.stack_size.write(writer)?;
            self.aux_value.write(writer)?;
            self.block_runtime_id.write(writer)?;
            self.user_data_buffer.write(writer)?;
        }
        Ok(())
    }
}

#[derive(PacketWrite)]
pub struct Entry {
    pub id: VarUInt,
    pub item: NetworkItemDescriptor,
    pub group_index: VarUInt,
}

pub struct ItemInstanceUserData {
    // https://mojang.github.io/bedrock-protocol-docs/html/ItemInstanceUserData.html
    //compound
    place_on_block_size: VarUInt,
    destroy_blocks_size: VarUInt,
}

impl PacketWrite for ItemInstanceUserData {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let mut buf = Vec::new();
        (-1i16).write(&mut buf)?;
        buf.extend_from_slice(&[1, 0]);
        self.place_on_block_size.write(&mut buf)?;
        self.destroy_blocks_size.write(&mut buf)?;
        VarUInt(buf.len() as u32).write(writer)?;
        writer.write_all(&buf)
    }
}

impl Default for ItemInstanceUserData {
    fn default() -> Self {
        Self {
            place_on_block_size: VarUInt(0),
            destroy_blocks_size: VarUInt(0),
        }
    }
}

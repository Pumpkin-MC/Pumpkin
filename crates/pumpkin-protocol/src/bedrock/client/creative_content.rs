use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::{
    bedrock::network_item::NetworkItemDescriptor, codec::var_uint::VarUInt, serial::PacketWrite,
};

#[packet(145)]
pub struct CCreativeContent<'a> {
    pub groups: &'a [CreativeGroupInfoPayload],
    pub entries: &'a [CreativeItemEntryPayload],
}

impl PacketWrite for CCreativeContent<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(self.groups.len() as u32).write(writer)?;
        for group in self.groups {
            group.write(writer)?;
        }

        VarUInt(self.entries.len() as u32).write(writer)?;
        for entry in self.entries {
            entry.write(writer)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CreativeCategory {
    All,
    Construction,
    Nature,
    Equipment,
    Items,
    ItemCommandOnly,
    Undefined,
}

impl PacketWrite for CreativeCategory {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match self {
            Self::Construction
            | Self::Nature
            | Self::Equipment
            | Self::Items
            | Self::ItemCommandOnly => (*self as u8).write(writer),
            _ => Err(Error::other("Invalid CreativeCategory to send")),
        }
    }
}

pub struct CreativeGroupInfoPayload {
    pub creative_category: CreativeCategory,
    pub name: String,

    // TODO: update inventory
    pub group_icon_item: NetworkItemDescriptor,
}

impl PacketWrite for CreativeGroupInfoPayload {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.creative_category.write(writer)?;
        self.name.write(writer)?;
        self.group_icon_item.write_item_instance(writer)
    }
}

pub struct CreativeItemEntryPayload {
    pub id: VarUInt,

    // TODO: update inventory
    pub item: NetworkItemDescriptor,

    pub group_index: VarUInt,
}

impl PacketWrite for CreativeItemEntryPayload {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.id.write(writer)?;
        self.item.write_item_instance(writer)?;
        self.group_index.write(writer)
    }
}

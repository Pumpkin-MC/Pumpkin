use std::io::Write;

use pumpkin_data::packet::clientbound::CONFIG_REGISTRY_DATA;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::{
    ClientPacket,
    codec::identifier::Identifier,
    ser::{NetworkWriteExt, WritingError},
};

#[packet(CONFIG_REGISTRY_DATA)]
pub struct CRegistryData<'a> {
    pub registry_id: &'a Identifier,
    pub entries: &'a [RegistryEntry],
}

impl<'a> CRegistryData<'a> {
    pub fn new(registry_id: &'a Identifier, entries: &'a [RegistryEntry]) -> Self {
        Self {
            registry_id,
            entries,
        }
    }
}

pub struct RegistryEntry {
    pub entry_id: Identifier,
    pub data: Option<Box<[u8]>>,
}

impl RegistryEntry {
    pub fn from_nbt(name: &str, nbt: &impl Serialize) -> Self {
        let mut data_buf = Vec::new();
        pumpkin_nbt::serializer::to_bytes_unnamed(nbt, &mut data_buf).unwrap();
        RegistryEntry {
            entry_id: Identifier::vanilla(name),
            data: Some(data_buf.into_boxed_slice()),
        }
    }
    pub fn from_nbt_custom(name: &str, nbt: &impl Serialize) -> Self {
        let mut data_buf = Vec::new();
        pumpkin_nbt::serializer::to_bytes_unnamed(nbt, &mut data_buf).unwrap();
        RegistryEntry {
            entry_id: Identifier::pumpkin(name),
            data: Some(data_buf.into_boxed_slice()),
        }
    }
}

impl ClientPacket for CRegistryData<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;
        write.write_identifier(self.registry_id)?;
        write.write_list::<RegistryEntry>(self.entries, |p, v| {
            p.write_identifier(&v.entry_id)?;
            p.write_option(&v.data, |p, v| p.write_slice(v))
        })
    }
}

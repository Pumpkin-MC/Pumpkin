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
    pub registry_id: Identifier<'a>,
    pub entries: &'a [RegistryEntry<'a>],
}

impl<'a> CRegistryData<'a> {
    pub fn new(registry_id: Identifier<'a>, entries: &'a [RegistryEntry]) -> Self {
        Self {
            registry_id,
            entries,
        }
    }
}

pub struct RegistryEntry<'a> {
    pub entry_id: Identifier<'a>,
    pub data: Option<Box<[u8]>>,
}

impl<'a> RegistryEntry<'a> {
    pub fn from_nbt(name: &'a str, nbt: &impl Serialize) -> Self {
        let mut data_buf = Vec::new();
        pumpkin_nbt::serializer::to_bytes_unnamed(nbt, &mut data_buf).unwrap();
        RegistryEntry {
            entry_id: Identifier::vanilla(name),
            data: Some(data_buf.into_boxed_slice()),
        }
    }
}

impl ClientPacket for CRegistryData<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;
        write.write_identifier(&self.registry_id)?;
        write.write_list::<RegistryEntry>(self.entries, |p, v| {
            p.write_identifier(&v.entry_id)?;
            p.write_option(&v.data, |p, v| p.write_slice(v))
        })
    }
}

use bytes::BufMut;
use pumpkin_data::packet::clientbound::CONFIG_REGISTRY_DATA;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::{ClientPacket, bytebuf::ByteBufMut, codec::identifier::Identifier};

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
}

impl ClientPacket for CRegistryData<'_> {
    fn write(&self, bytebuf: &mut impl BufMut) {
        bytebuf.put_identifier(self.registry_id);
        bytebuf.put_list::<RegistryEntry>(self.entries, |p, v| {
            p.put_identifier(&v.entry_id);
            p.put_option(&v.data, |p, v| p.put_slice(v));
        });
    }
}

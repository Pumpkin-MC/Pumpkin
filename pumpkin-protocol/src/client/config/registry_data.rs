use pumpkin_data::packet::clientbound::CONFIG_REGISTRY_DATA;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::{codec::identifier::Identifier, ser::network_serialize_no_prefix};

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct RegistryEntry {
    pub entry_id: Identifier,
    #[serde(serialize_with = "network_serialize_no_prefix")]
    pub data: Option<Box<[u8]>>,
}

// TODO: No unwraps
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

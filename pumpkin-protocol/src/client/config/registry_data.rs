use pumpkin_data::packet::clientbound::CONFIG_REGISTRY_DATA;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;
use serde::Serialize;

use crate::ser::network_serialize_no_prefix;

#[derive(Serialize)]
#[packet(CONFIG_REGISTRY_DATA)]
pub struct CRegistryData<'a> {
    pub registry_id: &'a ResourceLocation,
    pub entries: &'a [RegistryEntry],
}

impl<'a> CRegistryData<'a> {
    pub fn new(registry_id: &'a ResourceLocation, entries: &'a [RegistryEntry]) -> Self {
        Self {
            registry_id,
            entries,
        }
    }
}

#[derive(Serialize)]
pub struct RegistryEntry {
    pub entry_id: ResourceLocation,
    #[serde(serialize_with = "network_serialize_no_prefix")]
    pub data: Option<Box<[u8]>>,
}

// TODO: No unwraps
impl RegistryEntry {
    pub fn from_nbt(name: &str, nbt: &impl Serialize) -> Self {
        let mut data_buf = Vec::new();
        pumpkin_nbt::serializer::to_bytes_unnamed(nbt, &mut data_buf).unwrap();
        RegistryEntry {
            entry_id: ResourceLocation::vanilla(name),
            data: Some(data_buf.into_boxed_slice()),
        }
    }
    pub fn from_nbt_custom(name: &str, nbt: &impl Serialize) -> Self {
        let mut data_buf = Vec::new();
        pumpkin_nbt::serializer::to_bytes_unnamed(nbt, &mut data_buf).unwrap();
        RegistryEntry {
            entry_id: ResourceLocation::pumpkin(name),
            data: Some(data_buf.into_boxed_slice()),
        }
    }
}

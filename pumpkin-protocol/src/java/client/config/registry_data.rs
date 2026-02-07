use crate::ser::network_serialize_no_prefix;
use pumpkin_data::packet::clientbound::CONFIG_REGISTRY_DATA;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(CONFIG_REGISTRY_DATA)]
pub struct CRegistryData<'a> {
    pub registry_id: &'a ResourceLocation,
    pub entries: &'a [RegistryEntry],
}

impl<'a> CRegistryData<'a> {
    #[must_use]
    pub const fn new(registry_id: &'a ResourceLocation, entries: &'a [RegistryEntry]) -> Self {
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

impl RegistryEntry {
    #[must_use]
    pub const fn new(entry_id: ResourceLocation, data: Option<Box<[u8]>>) -> Self {
        Self { entry_id, data }
    }
}

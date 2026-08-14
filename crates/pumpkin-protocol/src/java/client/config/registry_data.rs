use std::io::Write;

use crate::{
    ClientPacket,
    codec::var_int::VarInt,
    ser::{NetworkWriteExt, WritingError},
};
use pumpkin_data::{packet::clientbound::CONFIG_REGISTRY_DATA, registry::RegistryEntryData};
use pumpkin_macros::java_packet;
use pumpkin_util::{resource_location::ResourceLocation, version::JavaMinecraftVersion};

#[java_packet(CONFIG_REGISTRY_DATA)]
pub struct CRegistryData<'a> {
    pub registry_id: &'a ResourceLocation,
    pub entries: &'a [RegistryEntryData],
}

impl<'a> CRegistryData<'a> {
    #[must_use]
    pub const fn new(registry_id: &'a ResourceLocation, entries: &'a [RegistryEntryData]) -> Self {
        Self {
            registry_id,
            entries,
        }
    }
}

impl ClientPacket for CRegistryData<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_string(&self.registry_id.clone())?;

        write.write_var_int(&VarInt(self.entries.len() as i32))?;
        for entry in self.entries {
            write.write_string(&entry.entry_id.clone())?;

            if let Some(data) = &entry.data {
                write.write_bool(true)?;
                write.write_all(data).map_err(WritingError::IoError)?;
            } else {
                write.write_bool(false)?;
            }
        }

        Ok(())
    }
}

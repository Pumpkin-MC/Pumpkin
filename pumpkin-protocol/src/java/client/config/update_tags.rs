use std::io::Write;

use crate::{ClientPacket, WritingError, ser::NetworkWriteExt};

use crate::codec::var_int::VarInt;
use pumpkin_data::{
    packet::clientbound::CONFIG_UPDATE_TAGS,
    tag::{RegistryKey, get_registry_key_tags},
};
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::version::MinecraftVersion;

#[java_packet(CONFIG_UPDATE_TAGS)]
pub struct CUpdateTags<'a> {
    pub tags: &'a [pumpkin_data::tag::RegistryKey],
}

impl<'a> CUpdateTags<'a> {
    #[must_use]
    pub const fn new(tags: &'a [RegistryKey]) -> Self {
        Self { tags }
    }
}

impl ClientPacket for CUpdateTags<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;
        write.write_list(self.tags, |p, registry_key| {
            p.write_resource_location(&ResourceLocation::vanilla(
                registry_key.identifier_string(),
            ))?;

            let values = get_registry_key_tags(*version, *registry_key).unwrap_or_else(|| {
                panic!(
                    "No tag data available for registry key '{}'",
                    registry_key.identifier_string()
                )
            });
            p.write_var_int(&values.len().try_into().map_err(|_| {
                WritingError::Message(format!("{} isn't representable as a VarInt", values.len()))
            })?)?;

            for (key, values) in values.entries() {
                // This is technically a `ResourceLocation` but same thing
                p.write_string_bounded(key, u16::MAX as usize)?;
                p.write_list(values.1, |p, id| p.write_var_int(&VarInt::from(*id)))?;
            }

            Ok(())
        })
    }
}

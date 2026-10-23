use std::io::Write;

use crate::packet::JavaPacket;
use crate::{ClientPacket, WritingError, ser::NetworkWriteExt};

use crate::codec::var_int::VarInt;
use pumpkin_data::{
    packet::clientbound::play::UPDATE_TAGS,
    tag::{RegistryKey, get_registry_key_tags},
};

pub struct CUpdateTagsPlay<'a> {
    pub tags: &'a [pumpkin_data::tag::RegistryKey],
}

impl JavaPacket for CUpdateTagsPlay<'_> {
    const PACKET_ID: i32 = UPDATE_TAGS.to_id();
}

impl<'a> CUpdateTagsPlay<'a> {
    #[must_use]
    pub const fn new(tags: &'a [RegistryKey]) -> Self {
        Self { tags }
    }
}

impl ClientPacket for CUpdateTagsPlay<'_> {
    fn write_packet_data(&self, mut write: impl Write) -> Result<(), WritingError> {
        let valid_keys: Vec<_> = self
            .tags
            .iter()
            .copied()
            .filter(RegistryKey::is_network_synced)
            .collect();

        write.write_list(&valid_keys, |p, &registry_key| {
            p.write_string(&format!("minecraft:{}", registry_key.identifier_string()))?;

            let Some(values) = get_registry_key_tags(registry_key) else {
                // no tags defined for that registry key
                // write an empty list and continue
                p.write_var_int(&VarInt::from(0))?;
                return Ok(());
            };
            p.write_var_int(&values.len().try_into().map_err(|_| {
                WritingError::Message(format!("{} isn't representable as a VarInt", values.len()))
            })?)?;

            for (key, values) in values.entries() {
                // This is technically a `ResourceLocation` but same thing
                p.write_string_bounded(key, u16::MAX as usize)?;
                p.write_list(values.1, |p, &id| p.write_var_int(&VarInt::from(id)))?;
            }

            Ok(())
        })
    }
}

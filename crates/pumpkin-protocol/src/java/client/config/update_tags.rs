use std::io::Write;

use crate::{ClientPacket, WritingError, ser::NetworkWriteExt};

use crate::codec::var_int::VarInt;
use pumpkin_data::{
    packet::clientbound::config::UPDATE_TAGS,
    tag::{RegistryKey, get_registry_key_tags},
};
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(UPDATE_TAGS)]
pub struct CUpdateTags<'a> {
    pub tags: &'a [pumpkin_data::tag::RegistryKey],
    /// Block tags from datapacks, already resolved to block ids. An entry with the
    /// same name as a built-in tag replaces it.
    pub block_tags: &'a [(String, Vec<u16>)],
}

impl<'a> CUpdateTags<'a> {
    #[must_use]
    pub const fn new(tags: &'a [RegistryKey]) -> Self {
        Self {
            tags,
            block_tags: &[],
        }
    }

    #[must_use]
    pub const fn with_block_tags(mut self, block_tags: &'a [(String, Vec<u16>)]) -> Self {
        self.block_tags = block_tags;
        self
    }
}

impl ClientPacket for CUpdateTags<'_> {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let valid_keys: Vec<_> = self
            .tags
            .iter()
            .copied()
            .filter(|key| key.is_valid_for_version(*version))
            .collect();

        write.write_list(&valid_keys, |p, &registry_key| {
            p.write_string(&format!("minecraft:{}", registry_key.identifier_string()))?;

            let Some(values) = get_registry_key_tags(*version, registry_key) else {
                // no tags defined for that registry key in this version
                // write an empty list and continue
                p.write_var_int(&VarInt::from(0))?;
                return Ok(());
            };
            let extra: &[(String, Vec<u16>)] = if registry_key == RegistryKey::Block {
                self.block_tags
            } else {
                &[]
            };
            let overridden = |key: &str| extra.iter().any(|(name, _)| name == key);
            let builtin: Vec<_> = values
                .entries()
                .filter(|(key, _)| !overridden(key))
                .collect();
            let count = builtin.len() + extra.len();
            p.write_var_int(&count.try_into().map_err(|_| {
                WritingError::Message(format!("{count} isn't representable as a VarInt"))
            })?)?;

            for (key, values) in builtin {
                // This is technically a `ResourceLocation` but same thing
                p.write_string_bounded(key, u16::MAX as usize)?;
                p.write_list(values.1, |p, &id| p.write_var_int(&VarInt::from(id)))?;
            }
            for (key, ids) in extra {
                p.write_string_bounded(key, u16::MAX as usize)?;
                p.write_list(ids, |p, &id| p.write_var_int(&VarInt::from(id)))?;
            }

            Ok(())
        })
    }
}

use std::io::Write;

use crate::packet::MultiVersionJavaPacket;
use crate::{ClientPacket, WritingError, ser::NetworkWriteExt};

use crate::codec::var_int::VarInt;
use pumpkin_data::{
    packet::clientbound::play::UPDATE_TAGS,
    tag::{RegistryKey, get_registry_key_tags},
};
use pumpkin_util::version::JavaMinecraftVersion;

pub struct CUpdateTagsPlay<'a> {
    pub tags: &'a [pumpkin_data::tag::RegistryKey],
}

impl MultiVersionJavaPacket for CUpdateTagsPlay<'_> {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        let id = UPDATE_TAGS.to_id(version);
        if id != -1 {
            return id;
        }
        #[allow(clippy::match_same_arms)]
        match version {
            JavaMinecraftVersion::V_1_20_5 => 0x78,
            JavaMinecraftVersion::V_1_20_3 => 0x74,
            JavaMinecraftVersion::V_1_20_2 => 0x70,
            JavaMinecraftVersion::V_1_20 | JavaMinecraftVersion::V_1_19_4 => 0x6E,
            JavaMinecraftVersion::V_1_19_3 => 0x6A,
            JavaMinecraftVersion::V_1_19_1 => 0x6B,
            JavaMinecraftVersion::V_1_19 => 0x68,
            JavaMinecraftVersion::V_1_18_2 | JavaMinecraftVersion::V_1_18 => 0x67,
            JavaMinecraftVersion::V_1_17_1 | JavaMinecraftVersion::V_1_17 => 0x66,
            JavaMinecraftVersion::V_1_16_4
            | JavaMinecraftVersion::V_1_16_3
            | JavaMinecraftVersion::V_1_16_2
            | JavaMinecraftVersion::V_1_16_1
            | JavaMinecraftVersion::V_1_16
            | JavaMinecraftVersion::V_1_14_4
            | JavaMinecraftVersion::V_1_14_3
            | JavaMinecraftVersion::V_1_14_2
            | JavaMinecraftVersion::V_1_14_1
            | JavaMinecraftVersion::V_1_14 => 0x5B,
            JavaMinecraftVersion::V_1_15_2
            | JavaMinecraftVersion::V_1_15_1
            | JavaMinecraftVersion::V_1_15 => 0x5C,
            JavaMinecraftVersion::V_1_13_2
            | JavaMinecraftVersion::V_1_13_1
            | JavaMinecraftVersion::V_1_13 => 0x55,
            _ => -1,
        }
    }
}

impl<'a> CUpdateTagsPlay<'a> {
    #[must_use]
    pub const fn new(tags: &'a [RegistryKey]) -> Self {
        Self { tags }
    }
}

impl ClientPacket for CUpdateTagsPlay<'_> {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_list(self.tags, |p, registry_key| {
            p.write_string(&format!("minecraft:{}", registry_key.identifier_string()))?;

            let Some(values) = get_registry_key_tags(*version, *registry_key) else {
                // no tags defined for that registry key in this version
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
                p.write_list(values.1, |p, id| p.write_var_int(&VarInt::from(*id)))?;
            }

            Ok(())
        })
    }
}

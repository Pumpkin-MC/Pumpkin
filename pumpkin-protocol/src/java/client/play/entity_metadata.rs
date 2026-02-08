use std::io::Cursor;

use pumpkin_data::{
    block_state_remap::remap_block_state_for_version, meta_data_type::MetaDataType,
    packet::clientbound::PLAY_SET_ENTITY_DATA, tracked_data::TrackedId,
};
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::{
    VarInt,
    ser::{NetworkWriteExt, WritingError, network_serialize_no_prefix, serializer},
};

const fn remap_metadata_type_id_for_version(
    type_id: i32,
    version: pumpkin_util::version::MinecraftVersion,
) -> Option<i32> {
    use pumpkin_util::version::MinecraftVersion;

    match version {
        // 1.21.7 / 1.21.8 (protocol 772) has a different metadata type table than 1.21.11.
        MinecraftVersion::V_1_21_7 => match type_id {
            // `compound_tag` exists at 16 in 1.21.7, so later ids are shifted.
            16..=27 => Some(type_id + 1),
            // 1.21.7 has no copper/weathering/profile/arm metadata types.
            28 | 33 | 34 | 37 | 38 => None,
            // `vector_3f` and `quaternion_f` are lower in 1.21.7.
            35 => Some(33),
            36 => Some(34),
            _ => Some(type_id),
        },
        // 1.21.9 / 1.21.10 (protocol 773) is close to latest but lacks some tail variants.
        MinecraftVersion::V_1_21_9 => match type_id {
            // Everything after that is shifted by one.
            29..=37 => Some(type_id - 1),
            // 1.21.11-only variants.
            28 | 38 => None,
            _ => Some(type_id),
        },
        _ => Some(type_id),
    }
}

/// Updates the "Data Tracker" values for an entity.
///
/// Entity Metadata (or `DataWatchers`) controls persistent visual states that
/// don't require a full packet to update, such as whether an entity is on fire,
/// crouching, glowing, or the custom name displayed above its head.
#[derive(Serialize)]
#[java_packet(PLAY_SET_ENTITY_DATA)]
pub struct CSetEntityMetadata {
    /// The Entity ID of the entity whose metadata is being updated.
    pub entity_id: VarInt,
    /// A serialized collection of metadata entries.
    /// Ends with a terminal byte (0xFF).
    #[serde(serialize_with = "network_serialize_no_prefix")]
    pub metadata: Box<[u8]>,
}

impl CSetEntityMetadata {
    #[must_use]
    pub const fn new(entity_id: VarInt, metadata: Box<[u8]>) -> Self {
        Self {
            entity_id,
            metadata,
        }
    }
}

pub struct Metadata<T> {
    index: TrackedId,
    r#type: VarInt,
    value: T,
}

impl<T> Metadata<T> {
    pub const fn new(index: TrackedId, r#type: MetaDataType, value: T) -> Self {
        Self {
            index,
            r#type: VarInt(r#type as i32),
            value,
        }
    }

    pub fn write<W: std::io::Write>(
        &self,
        mut writer: W,
        version: &pumpkin_util::version::MinecraftVersion,
    ) -> Result<(), WritingError>
    where
        T: Serialize,
    {
        let resolved_index = self.index.get(version);

        if resolved_index == 255 {
            return Ok(());
        }

        let Some(remapped_type_id) = remap_metadata_type_id_for_version(self.r#type.0, *version)
        else {
            // Metadata type does not exist in this protocol version.
            return Ok(());
        };

        writer.write_u8(resolved_index)?;
        writer.write_var_int(&VarInt(remapped_type_id))?;

        if self.r#type.0 == MetaDataType::BlockState as i32 {
            let mut serialized_value = Vec::new();
            {
                let mut serializer = serializer::Serializer::new(&mut serialized_value);
                self.value
                    .serialize(&mut serializer)
                    .map_err(|e| WritingError::Serde(e.to_string()))?;
            };

            let mut cursor = Cursor::new(serialized_value);
            let decoded_state = VarInt::decode(&mut cursor).map_err(|e| {
                WritingError::Message(format!("Failed to decode block state metadata: {e}"))
            })?;
            let remapped_state = u16::try_from(decoded_state.0).map_or(decoded_state, |state_id| {
                VarInt(i32::from(remap_block_state_for_version(state_id, *version)))
            });
            writer.write_var_int(&remapped_state)?;
            return Ok(());
        }

        let mut serializer = serializer::Serializer::new(&mut writer);
        self.value
            .serialize(&mut serializer)
            .map_err(|e| WritingError::Serde(e.to_string()))?;

        Ok(())
    }
}

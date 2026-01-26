use pumpkin_data::{
    meta_data_type::MetaDataType, packet::clientbound::PLAY_SET_ENTITY_DATA,
    tracked_data::TrackedId,
};
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::{
    VarInt,
    ser::{NetworkWriteExt, WritingError, network_serialize_no_prefix, serializer},
};

/// Updates the "Data Tracker" values for an entity.
///
/// Entity Metadata (or DataWatchers) controls persistent visual states that
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
    pub fn new(entity_id: VarInt, metadata: Box<[u8]>) -> Self {
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
    pub fn new(index: TrackedId, r#type: MetaDataType, value: T) -> Self {
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

        writer.write_u8(resolved_index)?;
        self.r#type.encode(&mut writer)?;

        let mut serializer = serializer::Serializer::new(&mut writer);
        self.value
            .serialize(&mut serializer)
            .map_err(|e| WritingError::Serde(e.to_string()))?;

        Ok(())
    }
}

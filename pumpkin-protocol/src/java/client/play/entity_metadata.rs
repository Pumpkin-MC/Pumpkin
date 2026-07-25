use std::io::{Cursor, Write};

use pumpkin_data::{
    block_state_remap::remap_block_state_for_version, item_id_remap::remap_item_id_for_version,
    meta_data_type::MetaDataType, packet::clientbound::PLAY_SET_ENTITY_DATA,
    tracked_data::TrackedId,
};
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ClientPacket, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

use super::particle::particle_id_for_version;

pub trait MetadataSerializer {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError>;
}

impl<T: MetadataSerializer + ?Sized> MetadataSerializer for &T {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        (*self).write_metadata(writer)
    }
}

/// Updates the "Data Tracker" values for an entity.
///
/// Entity Metadata (or `DataWatchers`) controls persistent visual states that
/// don't require a full packet to update, such as whether an entity is on fire,
/// crouching, glowing, or the custom name displayed above its head.
#[java_packet(PLAY_SET_ENTITY_DATA)]
pub struct CSetEntityMetadata {
    /// The Entity ID of the entity whose metadata is being updated.
    pub entity_id: VarInt,
    /// A serialized collection of metadata entries.
    /// Ends with a terminal byte (0xFF).
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

impl ClientPacket for CSetEntityMetadata {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        // 1. Entity ID
        write.write_var_int(&self.entity_id)?;

        write.write_slice(&self.metadata)
    }
}

pub struct Metadata<T> {
    pub index: TrackedId,
    pub r#type: MetaDataType,
    pub value: T,
}

impl<T> Metadata<T> {
    pub const fn new(index: TrackedId, r#type: MetaDataType, value: T) -> Self {
        Self {
            index,
            r#type,
            value,
        }
    }

    pub fn write<W: std::io::Write>(
        &self,
        mut writer: W,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError>
    where
        T: MetadataSerializer,
    {
        let resolved_index = self.index.get(version);

        if resolved_index == 255 {
            return Ok(());
        }

        let remapped_type_id = self.r#type.id(*version);
        if remapped_type_id < 0 {
            // Metadata type does not exist in this protocol version.
            return Ok(());
        }

        writer.write_u8(resolved_index)?;
        writer.write_var_int(&VarInt(remapped_type_id))?;

        if self.r#type == MetaDataType::BLOCK_STATE {
            let mut serialized_value = Vec::new();
            self.value.write_metadata(&mut serialized_value)?;

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

        if self.r#type == MetaDataType::ITEM_STACK {
            let mut serialized_value = Vec::new();
            self.value.write_metadata(&mut serialized_value)?;

            let mut cursor = Cursor::new(serialized_value);
            let item_count = VarInt::decode(&mut cursor).map_err(|e| {
                WritingError::Message(format!("Failed to decodeitem stack count: {e}"))
            })?;

            if item_count.0 <= 0 {
                writer.write_var_int(&item_count)?;
            } else {
                let item_id = VarInt::decode(&mut cursor)
                    .map_err(|e| WritingError::Message(format!("Failed to decode item id: {e}")))?;
                let remapped_id = u16::try_from(item_id.0)
                    .map_or(0, |id| remap_item_id_for_version(id, *version));
                writer.write_var_int(&item_count)?;
                writer.write_var_int(&VarInt(i32::from(remapped_id)))?;
                let remainder_start = cursor.position() as usize;
                let inner = cursor.into_inner();
                writer.write_slice(&inner[remainder_start..])?;
            }
            return Ok(());
        }

        if self.r#type == MetaDataType::PARTICLE {
            let mut serialized_value = Vec::new();
            self.value.write_metadata(&mut serialized_value)?;

            let mut cursor = Cursor::new(serialized_value);
            let particle_id = VarInt::decode(&mut cursor).map_err(|e| {
                WritingError::Message(format!("Failed to decode particle metadata: {e}"))
            })?;
            writer.write_var_int(&particle_id_for_version(particle_id, *version))?;

            let remainder_start = cursor.position() as usize;
            let inner = cursor.into_inner();
            writer.write_slice(&inner[remainder_start..])?;
            return Ok(());
        }

        self.value.write_metadata(&mut writer)?;

        Ok(())
    }
}

impl MetadataSerializer for bool {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_bool(*self)
    }
}

impl MetadataSerializer for i8 {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_i8(*self)
    }
}

impl MetadataSerializer for u8 {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_u8(*self)
    }
}

impl MetadataSerializer for i16 {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_i16(*self)
    }
}

impl MetadataSerializer for u16 {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_u16(*self)
    }
}

impl MetadataSerializer for i32 {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_i32(*self)
    }
}

impl MetadataSerializer for u32 {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_u32(*self)
    }
}

impl MetadataSerializer for f32 {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_f32(*self)
    }
}

impl MetadataSerializer for VarInt {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_var_int(self)
    }
}

impl MetadataSerializer for String {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_string(self)
    }
}

impl MetadataSerializer for pumpkin_util::text::TextComponent {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_slice(&self.encode())
    }
}

impl MetadataSerializer for Option<pumpkin_util::text::TextComponent> {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        if let Some(text) = self {
            writer.write_bool(true)?;
            writer.write_slice(&text.encode())?;
        } else {
            writer.write_bool(false)?;
        }
        Ok(())
    }
}

impl MetadataSerializer for crate::codec::item_stack_seralizer::ItemStackSerializer<'_> {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        self.write(writer)
    }
}

impl MetadataSerializer for Option<String> {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        if let Some(s) = self {
            writer.write_bool(true)?;
            writer.write_string(s)?;
        } else {
            writer.write_bool(false)?;
        }
        Ok(())
    }
}

impl MetadataSerializer for pumpkin_util::math::position::BlockPos {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_block_pos(self)
    }
}

impl MetadataSerializer for Option<pumpkin_util::math::position::BlockPos> {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        if let Some(pos) = self {
            writer.write_bool(true)?;
            writer.write_block_pos(pos)?;
        } else {
            writer.write_bool(false)?;
        }
        Ok(())
    }
}

impl MetadataSerializer for crate::codec::optional_int::OptionalInt {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        let val = self.0.map_or(0, |id| id + 1);
        writer.write_var_int(&VarInt(val))
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use pumpkin_data::{
        meta_data_type::MetaDataType, particle::Particle, tracked_data::TrackedData,
    };
    use pumpkin_util::version::JavaMinecraftVersion;

    use crate::{VarInt, ser::NetworkWriteExt};

    use super::{Metadata, MetadataSerializer};

    struct ParticleMetadata {
        particle_id: VarInt,
        data: [u8; 4],
    }

    impl MetadataSerializer for ParticleMetadata {
        fn write_metadata(
            &self,
            writer: &mut impl std::io::Write,
        ) -> Result<(), crate::WritingError> {
            writer.write_var_int(&self.particle_id)?;
            writer.write_slice(&self.data)
        }
    }

    fn encoded_particle(version: JavaMinecraftVersion) -> (VarInt, Vec<u8>) {
        let particle_data = [0x12, 0x34, 0x56, 0x78];
        let metadata = Metadata::new(
            TrackedData::PARTICLE,
            MetaDataType::PARTICLE,
            ParticleMetadata {
                particle_id: VarInt(Particle::ExplosionEmitter as i32),
                data: particle_data,
            },
        );
        let mut bytes = Vec::new();
        metadata.write(&mut bytes, &version).unwrap();

        assert_eq!(bytes[0], TrackedData::PARTICLE.get(&version));
        assert_eq!(bytes[1], MetaDataType::PARTICLE.id(version) as u8);

        let mut cursor = Cursor::new(&bytes[2..]);
        let particle_id = VarInt::decode(&mut cursor).unwrap();
        let mut remainder = Vec::new();
        cursor.read_to_end(&mut remainder).unwrap();
        (particle_id, remainder)
    }

    #[test]
    fn particle_metadata_id_remaps_for_1_21_11() {
        let (particle_id, data) = encoded_particle(JavaMinecraftVersion::V_1_21_11);

        assert_eq!(particle_id, VarInt(22));
        assert_eq!(data, [0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn particle_metadata_id_stays_latest_for_26_2() {
        let (particle_id, data) = encoded_particle(JavaMinecraftVersion::V_26_2);

        assert_eq!(particle_id, VarInt(29));
        assert_eq!(data, [0x12, 0x34, 0x56, 0x78]);
    }
}

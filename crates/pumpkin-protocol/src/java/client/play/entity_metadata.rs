use std::io::{Cursor, Write};

use pumpkin_data::{
    block_state_remap::remap_block_state_for_version,
    meta_data_type::MetaDataType,
    packet::clientbound::play::SET_ENTITY_DATA,
    tracked_data::{TrackedData, TrackedId},
};
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ClientPacket, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

use super::particle::particle_id_for_version;

pub trait MetadataSerializer {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError>;
}

impl<T: MetadataSerializer + ?Sized> MetadataSerializer for &T {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        (*self).write_metadata(writer, version)
    }
}

/// Updates the "Data Tracker" values for an entity.
///
/// Entity Metadata (or `DataWatchers`) controls persistent visual states that
/// don't require a full packet to update, such as whether an entity is on fire,
/// crouching, glowing, or the custom name displayed above its head.
#[java_packet(SET_ENTITY_DATA)]
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
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        // 1. Entity ID
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i32_be(self.entity_id.0)?;
        } else {
            write.write_var_int(&self.entity_id)?;
        }

        write.write_slice(&self.metadata)
    }
}

impl<'a> crate::ServerPacket<'a> for CSetEntityMetadata {
    fn read(
        bytebuf: &mut &'a [u8],
        version: &JavaMinecraftVersion,
    ) -> Result<Self, crate::ser::ReadingError> {
        use crate::ser::{NetworkReadExt, NetworkReadSliceExt};
        let entity_id = if *version <= JavaMinecraftVersion::V_1_7_6 {
            VarInt(bytebuf.get_i32_be()?)
        } else {
            bytebuf.get_var_int()?
        };
        let metadata = bytebuf.read_remaining_slice_borrowed(usize::MAX)?;
        Ok(Self {
            entity_id,
            metadata: metadata.into(),
        })
    }
}

pub struct Metadata<T> {
    pub index: TrackedId,
    pub r#type: MetaDataType,
    pub value: T,
}

impl<T> Metadata<T> {
    pub const fn new(tracked: TrackedData, value: T) -> Self {
        Self {
            index: tracked.id,
            r#type: tracked.r#type,
            value,
        }
    }

    pub const fn new_raw(index: TrackedId, r#type: MetaDataType, value: T) -> Self {
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
            self.value.write_metadata(&mut serialized_value, version)?;

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
            // Delegates to ItemStackSerializer::write_with_version, which remaps both the item
            // id and every data component id for `version` - unlike the other branches here,
            // there's no separate post-processing step needed since the item's own version-aware
            // writer already produces correct bytes end to end.
            self.value.write_metadata(&mut writer, version)?;
            return Ok(());
        }

        if self.r#type == MetaDataType::PARTICLE {
            let mut serialized_value = Vec::new();
            self.value.write_metadata(&mut serialized_value, version)?;

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

        self.value.write_metadata(&mut writer, version)?;

        Ok(())
    }
}

impl MetadataSerializer for bool {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_bool(*self)
    }
}

impl MetadataSerializer for i8 {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_i8(*self)
    }
}

impl MetadataSerializer for u8 {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_u8(*self)
    }
}

impl MetadataSerializer for i16 {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_i16(*self)
    }
}

impl MetadataSerializer for u16 {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_u16(*self)
    }
}

impl MetadataSerializer for i32 {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_i32(*self)
    }
}

impl MetadataSerializer for u32 {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_u32(*self)
    }
}

impl MetadataSerializer for f32 {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_f32(*self)
    }
}

impl MetadataSerializer for VarInt {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_var_int(self)
    }
}

impl MetadataSerializer for String {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_string(self)
    }
}

impl MetadataSerializer for pumpkin_util::text::TextComponent {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version < JavaMinecraftVersion::V_1_20_3 {
            let json = self.to_json_for_version(version);
            writer.write_string(&json)
        } else {
            writer.write_slice(&self.encode_for_version(version))
        }
    }
}

impl MetadataSerializer for Option<pumpkin_util::text::TextComponent> {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if let Some(text) = self {
            writer.write_bool(true)?;
            text.write_metadata(writer, version)?;
        } else {
            writer.write_bool(false)?;
        }
        Ok(())
    }
}

impl MetadataSerializer for crate::codec::item_stack_seralizer::ItemStackSerializer<'_> {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        self.write_with_version(writer, version)
    }
}

impl MetadataSerializer for Option<String> {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
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
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_block_pos(self, version)
    }
}

impl MetadataSerializer for Option<pumpkin_util::math::position::BlockPos> {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if let Some(pos) = self {
            writer.write_bool(true)?;
            writer.write_block_pos(pos, version)?;
        } else {
            writer.write_bool(false)?;
        }
        Ok(())
    }
}

impl MetadataSerializer for crate::codec::optional_int::OptionalInt {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let val = self.0.map_or(0, |id| id + 1);
        writer.write_var_int(&VarInt(val))
    }
}

impl MetadataSerializer for uuid::Uuid {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        writer.write_uuid(self)
    }
}

impl MetadataSerializer for Option<uuid::Uuid> {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if let Some(uuid) = self {
            writer.write_bool(true)?;
            writer.write_uuid(uuid)?;
        } else {
            writer.write_bool(false)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use pumpkin_data::{meta_data_type::MetaDataType, particle::Particle};
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
            _version: &JavaMinecraftVersion,
        ) -> Result<(), crate::WritingError> {
            writer.write_var_int(&self.particle_id)?;
            writer.write_slice(&self.data)
        }
    }

    fn encoded_particle(version: JavaMinecraftVersion) -> (VarInt, Vec<u8>) {
        let particle_data = [0x12, 0x34, 0x56, 0x78];
        let metadata = Metadata::new(
            pumpkin_data::tracked_data::area_effect_cloud::DATA_PARTICLE,
            ParticleMetadata {
                particle_id: VarInt(Particle::ExplosionEmitter as i32),
                data: particle_data,
            },
        );
        let mut bytes = Vec::new();
        metadata.write(&mut bytes, &version).unwrap();

        assert_eq!(
            bytes[0],
            pumpkin_data::tracked_data::area_effect_cloud::DATA_PARTICLE.get(&version)
        );
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

    // Regression test: entity metadata of type ITEM_STACK (e.g. a dropped ItemEntity's visual)
    // used to serialize the item generically and only remap the leading item id, leaving data
    // component ids inside at the current (26.1+) numbering regardless of the recipient's
    // version - breaking decode for any pre-26.1 client viewing an item with a real component
    // patch (e.g. a firework rocket with actual flight_duration/explosions data).
    #[test]
    fn item_stack_metadata_uses_legacy_component_id_for_1_21_11() {
        use crate::codec::item_stack_seralizer::ItemStackSerializer;
        use pumpkin_data::data_component::DataComponent;
        use pumpkin_data::data_component_impl::{DataComponentImpl, FireworksImpl};
        use pumpkin_data::item::Item;
        use pumpkin_data::item_stack::ItemStack;

        let fireworks = FireworksImpl::new(3, Vec::new());
        let mut stack = ItemStack::new(1, &Item::FIREWORK_ROCKET);
        stack
            .patch
            .push((DataComponent::Fireworks, Some(fireworks.to_dyn())));

        let metadata = Metadata::new(
            pumpkin_data::tracked_data::item::ITEM,
            ItemStackSerializer::from(stack),
        );
        let mut bytes = Vec::new();
        metadata
            .write(&mut bytes, &JavaMinecraftVersion::V_1_21_11)
            .unwrap();

        // index byte, type id varint, then item_count, item_id, num_to_add, num_to_remove,
        // component id.
        let mut cursor = Cursor::new(&bytes[2..]);
        VarInt::decode(&mut cursor).unwrap(); // item_count
        VarInt::decode(&mut cursor).unwrap(); // item_id
        VarInt::decode(&mut cursor).unwrap(); // num_to_add
        VarInt::decode(&mut cursor).unwrap(); // num_to_remove
        let mut remainder = Vec::new();
        cursor.read_to_end(&mut remainder).unwrap();

        assert_eq!(
            remainder[0], 67,
            "fireworks component in entity metadata must use legacy id 67 for a 1.21.11 \
             recipient, not the current id 69"
        );
    }
}

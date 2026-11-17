use pumpkin_data::packet::serverbound::play::SET_STRUCTURE_BLOCK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::{var_int::VarInt, var_long::VarLong},
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

#[java_packet(SET_STRUCTURE_BLOCK)]
pub struct SSetStructureBlock<'a> {
    pub location: BlockPos,
    pub action: VarInt,
    pub mode: VarInt,
    pub name: &'a str,
    pub offset_x: i8,
    pub offset_y: i8,
    pub offset_z: i8,
    pub size_x: u8,
    pub size_y: u8,
    pub size_z: u8,
    pub mirror: VarInt,
    pub rotation: VarInt,
    pub metadata: &'a str,
    pub integrity: f32,
    pub seed: VarLong,
    pub flags: u8,
}

impl SSetStructureBlock<'_> {
    pub const ACTION_UPDATE_DATA: i32 = 0;
    pub const ACTION_SAVE_AREA: i32 = 1;
    pub const ACTION_LOAD_AREA: i32 = 2;
    pub const ACTION_SCAN_AREA: i32 = 3;

    pub const MODE_SAVE: i32 = 0;
    pub const MODE_LOAD: i32 = 1;
    pub const MODE_CORNER: i32 = 2;
    pub const MODE_DATA: i32 = 3;

    pub const FLAG_IGNORE_ENTITIES: u8 = 0x01;
    pub const FLAG_SHOW_AIR: u8 = 0x02;
    pub const FLAG_SHOW_BOUNDING_BOX: u8 = 0x04;
    pub const FLAG_STRICT: u8 = 0x08;

    #[must_use]
    pub const fn ignore_entities(&self) -> bool {
        (self.flags & Self::FLAG_IGNORE_ENTITIES) != 0
    }

    #[must_use]
    pub const fn show_air(&self) -> bool {
        (self.flags & Self::FLAG_SHOW_AIR) != 0
    }

    #[must_use]
    pub const fn show_bounding_box(&self) -> bool {
        (self.flags & Self::FLAG_SHOW_BOUNDING_BOX) != 0
    }

    #[must_use]
    pub const fn strict(&self) -> bool {
        (self.flags & Self::FLAG_STRICT) != 0
    }
}

impl<'a> ServerPacket<'a> for SSetStructureBlock<'a> {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let location = bytebuf.get_block_pos(version)?;
        let action = bytebuf.get_var_int()?;
        let mode = bytebuf.get_var_int()?;
        let name = bytebuf.get_str_borrowed()?;
        let offset_x = bytebuf.get_i8()?.clamp(-48, 48);
        let offset_y = bytebuf.get_i8()?.clamp(-48, 48);
        let offset_z = bytebuf.get_i8()?.clamp(-48, 48);
        let size_x = bytebuf.get_i8()?.clamp(0, 48) as u8;
        let size_y = bytebuf.get_i8()?.clamp(0, 48) as u8;
        let size_z = bytebuf.get_i8()?.clamp(0, 48) as u8;
        let mirror = bytebuf.get_var_int()?;
        let rotation = bytebuf.get_var_int()?;
        let metadata = bytebuf.get_str_bounded_borrowed(128)?;
        let raw_integrity = bytebuf.get_f32()?;
        let integrity = if raw_integrity.is_nan() {
            0.0
        } else {
            raw_integrity.clamp(0.0, 1.0)
        };
        let seed = bytebuf.get_var_long()?;
        let flags = bytebuf.get_u8()?;

        Ok(Self {
            location,
            action,
            mode,
            name,
            offset_x,
            offset_y,
            offset_z,
            size_x,
            size_y,
            size_z,
            mirror,
            rotation,
            metadata,
            integrity,
            seed,
            flags,
        })
    }
}

impl crate::ClientPacket for SSetStructureBlock<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_block_pos(&self.location, version)?;
        write.write_var_int(&self.action)?;
        write.write_var_int(&self.mode)?;
        write.write_string(self.name)?;
        write.write_i8(self.offset_x)?;
        write.write_i8(self.offset_y)?;
        write.write_i8(self.offset_z)?;
        write.write_u8(self.size_x)?;
        write.write_u8(self.size_y)?;
        write.write_u8(self.size_z)?;
        write.write_var_int(&self.mirror)?;
        write.write_var_int(&self.rotation)?;
        write.write_string(self.metadata)?;
        write.write_f32_be(self.integrity)?;
        write.write_var_long(&self.seed)?;
        write.write_u8(self.flags)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SSetStructureBlock;
    use crate::{ClientPacket, ServerPacket, codec::var_int::VarInt, codec::var_long::VarLong};
    use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

    #[test]
    fn set_structure_block_clamps_bounds_and_decodes_strict_flag() {
        let raw = SSetStructureBlock {
            location: BlockPos::new(10, 64, -5),
            action: VarInt(SSetStructureBlock::ACTION_SAVE_AREA),
            mode: VarInt(SSetStructureBlock::MODE_SAVE),
            name: "test:box",
            offset_x: -100,
            offset_y: 12,
            offset_z: 90,
            size_x: 250, // -6 as signed i8 -> clamped to 0
            size_y: 60,  // clamped to 48
            size_z: 16,
            mirror: VarInt(1),
            rotation: VarInt(2),
            metadata: "chest_loot",
            integrity: 1.5,
            seed: VarLong(42),
            flags: SSetStructureBlock::FLAG_IGNORE_ENTITIES | SSetStructureBlock::FLAG_STRICT,
        };

        let mut buf = Vec::new();
        raw.write_packet_data(&mut buf, &JavaMinecraftVersion::V_26_3)
            .unwrap();

        let mut slice = buf.as_slice();
        let decoded = SSetStructureBlock::read(&mut slice, &JavaMinecraftVersion::V_26_3).unwrap();
        assert!(slice.is_empty());
        assert_eq!(decoded.location, BlockPos::new(10, 64, -5));
        assert_eq!(decoded.offset_x, -48);
        assert_eq!(decoded.offset_y, 12);
        assert_eq!(decoded.offset_z, 48);
        assert_eq!(decoded.size_x, 0);
        assert_eq!(decoded.size_y, 48);
        assert_eq!(decoded.size_z, 16);
        assert!((decoded.integrity - 1.0).abs() < f32::EPSILON);
        assert!(decoded.ignore_entities());
        assert!(decoded.strict());
        assert!(!decoded.show_air());
        assert!(!decoded.show_bounding_box());
    }
}

use crate::{
    ServerPacket,
    codec::item_stack_seralizer::ItemStackSerializer,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::USE_ITEM_ON;
use pumpkin_macros::java_packet;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(USE_ITEM_ON)]
pub struct SUseItemOn {
    pub hand: VarInt,
    pub position: BlockPos,
    pub face: VarInt,
    pub cursor_pos: Vector3<f32>,
    pub inside_block: bool,
    pub is_against_world_border: bool,
    pub sequence: VarInt,
}

impl<'a> ServerPacket<'a> for SUseItemOn {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_14 {
            let hand = bytebuf.get_var_int()?;
            let position = bytebuf.get_block_pos(version)?;
            let face = bytebuf.get_var_int()?;
            let cursor_pos = Vector3::new(
                bytebuf.get_f32_be()?,
                bytebuf.get_f32_be()?,
                bytebuf.get_f32_be()?,
            );
            let inside_block = bytebuf.get_bool()?;
            let (is_against_world_border, sequence) = if *version >= JavaMinecraftVersion::V_1_19 {
                let is_against_world_border = if *version >= JavaMinecraftVersion::V_1_21_2 {
                    bytebuf.get_bool()?
                } else {
                    false
                };
                let sequence = bytebuf.get_var_int()?;
                (is_against_world_border, sequence)
            } else {
                (false, VarInt(0))
            };

            Ok(Self {
                hand,
                position,
                face,
                cursor_pos,
                inside_block,
                is_against_world_border,
                sequence,
            })
        } else {
            let position = if *version <= JavaMinecraftVersion::V_1_7_6 {
                let x = bytebuf.get_i32_be()?;
                let y = bytebuf.get_u8()? as i32;
                let z = bytebuf.get_i32_be()?;
                BlockPos::new(x, y, z)
            } else {
                bytebuf.get_block_pos(version)?
            };

            let (face, hand) = if *version >= JavaMinecraftVersion::V_1_9 {
                let face = bytebuf.get_var_int()?;
                let hand = bytebuf.get_var_int()?;
                (face, hand)
            } else {
                let face = VarInt(i32::from(bytebuf.get_u8()?));
                let _item_stack = ItemStackSerializer::read_with_version(bytebuf, version)?;
                (face, VarInt(0))
            };

            let cursor_pos = if *version >= JavaMinecraftVersion::V_1_11 {
                Vector3::new(
                    bytebuf.get_f32_be()?,
                    bytebuf.get_f32_be()?,
                    bytebuf.get_f32_be()?,
                )
            } else {
                Vector3::new(
                    f32::from(bytebuf.get_u8()?) / 16.0,
                    f32::from(bytebuf.get_u8()?) / 16.0,
                    f32::from(bytebuf.get_u8()?) / 16.0,
                )
            };

            Ok(Self {
                hand,
                position,
                face,
                cursor_pos,
                inside_block: false,
                is_against_world_border: false,
                sequence: VarInt(0),
            })
        }
    }
}

impl crate::ClientPacket for SUseItemOn {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        if *version >= JavaMinecraftVersion::V_1_14 {
            write.write_var_int(&self.hand)?;
            write.write_block_pos(&self.position, version)?;
            write.write_var_int(&self.face)?;
            write.write_f32_be(self.cursor_pos.x)?;
            write.write_f32_be(self.cursor_pos.y)?;
            write.write_f32_be(self.cursor_pos.z)?;
            write.write_bool(self.inside_block)?;
            if *version >= JavaMinecraftVersion::V_1_19 {
                if *version >= JavaMinecraftVersion::V_1_21_2 {
                    write.write_bool(self.is_against_world_border)?;
                }
                write.write_var_int(&self.sequence)?;
            }
        } else {
            if *version <= JavaMinecraftVersion::V_1_7_6 {
                write.write_i32_be(self.position.0.x)?;
                write.write_u8(self.position.0.y.clamp(0, 255) as u8)?;
                write.write_i32_be(self.position.0.z)?;
            } else {
                write.write_block_pos(&self.position, version)?;
            }
            if *version >= JavaMinecraftVersion::V_1_9 {
                write.write_var_int(&self.face)?;
                write.write_var_int(&self.hand)?;
            } else {
                write.write_u8(self.face.0.clamp(0, 255) as u8)?;
                // heldItem (empty slot: -1 as i16)
                write.write_i16_be(-1)?;
            }
            if *version >= JavaMinecraftVersion::V_1_11 {
                write.write_f32_be(self.cursor_pos.x)?;
                write.write_f32_be(self.cursor_pos.y)?;
                write.write_f32_be(self.cursor_pos.z)?;
            } else {
                write.write_u8((self.cursor_pos.x * 16.0).round().clamp(0.0, 255.0) as u8)?;
                write.write_u8((self.cursor_pos.y * 16.0).round().clamp(0.0, 255.0) as u8)?;
                write.write_u8((self.cursor_pos.z * 16.0).round().clamp(0.0, 255.0) as u8)?;
            }
        }
        Ok(())
    }
}

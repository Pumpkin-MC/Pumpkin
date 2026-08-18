use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_USE_ITEM_ON;
use pumpkin_macros::java_packet;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLAY_USE_ITEM_ON)]
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
        let hand = bytebuf.get_var_int()?;
        let position = BlockPos::from_i64(bytebuf.get_i64_be()?);
        let face = bytebuf.get_var_int()?;
        let cursor_pos = Vector3::new(
            bytebuf.get_f32_be()?,
            bytebuf.get_f32_be()?,
            bytebuf.get_f32_be()?,
        );
        let inside_block = bytebuf.get_bool()?;
        let is_against_world_border = if version >= &JavaMinecraftVersion::V_1_21_5 {
            bytebuf.get_bool()?
        } else {
            false
        };
        let sequence = bytebuf.get_var_int()?;

        Ok(Self {
            hand,
            position,
            face,
            cursor_pos,
            inside_block,
            is_against_world_border,
            sequence,
        })
    }
}

impl crate::ClientPacket for SUseItemOn {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.hand)?;
        write.write_block_pos(&self.position)?;
        write.write_var_int(&self.face)?;
        write.write_f32_be(self.cursor_pos.x)?;
        write.write_f32_be(self.cursor_pos.y)?;
        write.write_f32_be(self.cursor_pos.z)?;
        write.write_bool(self.inside_block)?;
        if version >= &JavaMinecraftVersion::V_1_21_5 {
            write.write_bool(self.is_against_world_border)?;
        }
        write.write_var_int(&self.sequence)?;
        Ok(())
    }
}

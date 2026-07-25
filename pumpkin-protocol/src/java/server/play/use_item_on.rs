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
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            hand: bytebuf.get_var_int()?,
            position: BlockPos::from_i64(bytebuf.get_i64_be()?),
            face: bytebuf.get_var_int()?,
            cursor_pos: Vector3::new(
                bytebuf.get_f32_be()?,
                bytebuf.get_f32_be()?,
                bytebuf.get_f32_be()?,
            ),
            inside_block: bytebuf.get_bool()?,
            is_against_world_border: bytebuf.get_bool()?,
            sequence: bytebuf.get_var_int()?,
        })
    }
}

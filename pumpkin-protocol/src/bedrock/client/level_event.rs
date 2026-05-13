use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(25)]
pub struct CLevelEvent {
    pub event_id: VarInt,
    pub position: Vector3<f32>,
    pub data: VarInt,
}

#[repr(i32)]
pub enum LevelEvent {
    // There are hundreds of these, adding only what we need for now
    BlockStartBreak = 3600,
    BlockStopBreak = 3601,
    BlockUpdateBreak = 3602,
}

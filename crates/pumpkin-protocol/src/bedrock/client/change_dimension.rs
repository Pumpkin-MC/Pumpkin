use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(61)]
pub struct CChangeDimension {
    pub dimension: VarInt,
    pub position: Vector3<f32>,
    pub respawn: bool,
    pub has_loading_screen_id: bool,
}

impl CChangeDimension {
    #[must_use]
    pub const fn new(dimension: i32, position: Vector3<f32>, respawn: bool) -> Self {
        Self {
            dimension: VarInt(dimension),
            position,
            respawn,
            has_loading_screen_id: false,
        }
    }
}

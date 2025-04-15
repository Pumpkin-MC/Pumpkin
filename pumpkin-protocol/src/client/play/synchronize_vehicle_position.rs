use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_TELEPORT_ENTITY;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    ClientPacket, PositionFlag, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

// https://minecraft.wiki/w/Java_Edition_protocol#Synchronize_Vehicle_Position
// Badly documented and confusing packet imo
#[packet(PLAY_TELEPORT_ENTITY)]
pub struct CSynchronizeVehiclePosition<'a> {
    entity_id: VarInt,
    position: Vector3<f64>,
    delta: Vector3<f64>,
    yaw: f32,
    pitch: f32,
    relatives: &'a [PositionFlag],
    on_ground: bool,
}

impl<'a> CSynchronizeVehiclePosition<'a> {
    pub fn new(
        entity_id: VarInt,
        position: Vector3<f64>,
        delta: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        relatives: &'a [PositionFlag],
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            position,
            delta,
            yaw,
            pitch,
            relatives,
            on_ground,
        }
    }
}

// TODO: Do we need a custom impl?
impl ClientPacket for CSynchronizeVehiclePosition<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.entity_id)?;
        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;
        write.write_f64_be(self.delta.x)?;
        write.write_f64_be(self.delta.y)?;
        write.write_f64_be(self.delta.z)?;
        write.write_f32_be(self.yaw)?;
        write.write_f32_be(self.pitch)?;
        // not sure about that
        write.write_i32_be(PositionFlag::get_bitfield(self.relatives))?;
        write.write_bool(self.on_ground)
    }
}

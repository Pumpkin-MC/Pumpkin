use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_PLAYER_POSITION;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    ClientPacket, PositionFlag, ServerPacket, VarInt, WritingError, ser::NetworkReadExt,
    ser::NetworkWriteExt,
};

#[packet(PLAY_PLAYER_POSITION)]
pub struct CPlayerPosition {
    pub teleport_id: VarInt,
    pub position: Vector3<f64>,
    pub delta: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    pub releatives: Vec<PositionFlag>,
}

impl CPlayerPosition {
    pub fn new(
        teleport_id: VarInt,
        position: Vector3<f64>,
        delta: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        releatives: Vec<PositionFlag>,
    ) -> Self {
        Self {
            teleport_id,
            position,
            delta,
            yaw,
            pitch,
            releatives,
        }
    }
}

// TODO: Do we need a custom impl?
impl ClientPacket for CPlayerPosition {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.teleport_id)?;
        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;
        write.write_f64_be(self.delta.x)?;
        write.write_f64_be(self.delta.y)?;
        write.write_f64_be(self.delta.z)?;
        write.write_f32_be(self.yaw)?;
        write.write_f32_be(self.pitch)?;
        // not sure about that
        write.write_i32_be(PositionFlag::get_bitfield(self.releatives.as_slice()))
    }
}

impl ServerPacket for CPlayerPosition {
    fn read(mut read: impl std::io::Read) -> Result<Self, crate::ser::ReadingError> {
        Ok(Self {
            teleport_id: read.get_var_int()?,
            // TODO
            position: Vector3::new(0.0, 0.0, 0.0),
            delta: Vector3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            releatives: Vec::new(),
        })
    }
}

use bytes::BufMut;
use pumpkin_data::packet::clientbound::PLAY_TELEPORT_ENTITY;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{ClientPacket, PositionFlag, VarInt, bytebuf::ByteBufMut};

#[packet(PLAY_TELEPORT_ENTITY)]
pub struct CTeleportEntity<'a> {
    entity_id: VarInt,
    position: Vector3<f64>,
    delta: Vector3<f64>,
    yaw: f32,
    pitch: f32,
    releatives: &'a [PositionFlag],
    on_ground: bool,
}

impl<'a> CTeleportEntity<'a> {
    pub fn new(
        entity_id: VarInt,
        position: Vector3<f64>,
        delta: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        releatives: &'a [PositionFlag],
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            position,
            delta,
            yaw,
            pitch,
            releatives,
            on_ground,
        }
    }
}

impl ClientPacket for CTeleportEntity<'_> {
    fn write(&self, bytebuf: &mut impl BufMut) {
        bytebuf.put_var_int(&self.entity_id);
        bytebuf.put_f64(self.position.x);
        bytebuf.put_f64(self.position.y);
        bytebuf.put_f64(self.position.z);
        bytebuf.put_f64(self.delta.x);
        bytebuf.put_f64(self.delta.y);
        bytebuf.put_f64(self.delta.z);
        bytebuf.put_f32(self.yaw);
        bytebuf.put_f32(self.pitch);
        // not sure about that
        bytebuf.put_i32(PositionFlag::get_bitfield(self.releatives));
        bytebuf.put_bool(self.on_ground);
    }
}

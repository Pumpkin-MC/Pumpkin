use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Sent by the server to spawn a visual particle effect at a specific 3D location in the world.
///
/// Packet ID: `123`
/// Ref: <https://mojang.github.io/bedrock-protocol-docs/html/LevelSoundEventPacket.html>
#[derive(PacketWrite)]
#[packet(123)]
pub struct CLevelSoundEvent {
    pub sound_id: String,
    pub position: Vector3<f32>,
    pub extra_data: VarInt,
    pub entity_type: String,
    pub is_baby_mob: bool,
    pub is_global: bool,
    pub actor_unique_id: i64,
    pub fire_at_position: Option<Vector3<f32>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_sound_event_uses_cereal_payload() {
        assert_eq!(<CLevelSoundEvent as crate::Packet>::PACKET_ID, 123);

        let packet = CLevelSoundEvent {
            sound_id: "test".into(),
            position: Vector3::new(1.0, 2.0, 3.0),
            extra_data: VarInt(-1),
            entity_type: "actor".into(),
            is_baby_mob: true,
            is_global: false,
            actor_unique_id: 42,
            fire_at_position: Some(Vector3::new(4.0, 5.0, 6.0)),
        };
        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();

        assert_eq!(&encoded[..5], b"\x04test");
        assert_eq!(encoded[17], 1); // Zig-zag encoded -1.
        assert_eq!(&encoded[18..24], b"\x05actor");
        assert_eq!(&encoded[24..26], &[1, 0]);
        assert_eq!(&encoded[26..34], &42i64.to_le_bytes());
        assert_eq!(encoded[34], 1);
        assert_eq!(encoded.len(), 47);
    }
}

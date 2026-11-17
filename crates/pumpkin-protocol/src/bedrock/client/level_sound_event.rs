// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Plays a sound effect at a designated world position or associated with an actor.
#[derive(PacketWrite)]
#[packet(123)]
pub struct CLevelSoundEvent {
    /// Canonical identifier string for the sound event.
    pub sound_event: String,
    /// World coordinates where the sound source is located.
    pub position: Vector3<f32>,
    /// Auxiliary integer parameter used by specific sound effects.
    pub data: VarInt,
    /// Identifier of the entity generating the sound (e.g. `minecraft:skeleton`).
    pub actor_identifier: String,
    /// Whether the sound pitch/variant should reflect a juvenile mob variant.
    pub is_baby: bool,
    /// Whether the sound plays globally at full volume without spatial falloff.
    pub is_global: bool,
    /// Unique identifier of the entity emitting the sound.
    pub actor_unique_id: i64,
    /// Optional explicit override position where the event should trigger.
    pub fire_at_position: Option<Vector3<f32>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_sound_event_uses_cereal_payload() {
        assert_eq!(<CLevelSoundEvent as crate::Packet>::PACKET_ID, 123);

        let packet = CLevelSoundEvent {
            sound_event: "test".into(),
            position: Vector3::new(1.0, 2.0, 3.0),
            data: VarInt(-1),
            actor_identifier: "actor".into(),
            is_baby: true,
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

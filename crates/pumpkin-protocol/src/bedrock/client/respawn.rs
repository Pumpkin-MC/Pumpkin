use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{bedrock::respawn::RespawnState, codec::var_ulong::VarULong, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(45)]
pub struct CRespawn {
    pub position: Vector3<f32>,
    pub state: RespawnState,
    pub player_runtime_id: VarULong,
}

impl CRespawn {
    #[must_use]
    pub const fn new(
        position: Vector3<f32>,
        state: RespawnState,
        player_runtime_id: VarULong,
    ) -> Self {
        Self {
            position,
            state,
            player_runtime_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bedrock::server::respawn::SRespawn, serial::PacketRead};

    #[test]
    fn respawn_packet_roundtrip() {
        let packet = CRespawn::new(
            Vector3::new(1.5, 64.0, -2.25),
            RespawnState::ReadyToSpawn,
            VarULong(42),
        );
        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();

        let decoded = SRespawn::read(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded.position, packet.position);
        assert_eq!(decoded.state, packet.state);
        assert_eq!(decoded.player_runtime_id.0, packet.player_runtime_id.0);
    }
}

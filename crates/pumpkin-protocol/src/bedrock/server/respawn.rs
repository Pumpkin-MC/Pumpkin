// Last verified for v2169

use std::io::{Error, ErrorKind, Read, Write};

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    codec::var_ulong::VarULong,
    serial::{PacketRead, PacketWrite},
};

#[derive(PacketRead, PacketWrite)]
#[packet(45)]
pub struct SRespawn {
    pub position: Vector3<f32>,
    pub state: RespawnState,
    pub player_runtime_id: VarULong,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RespawnState {
    SearchingForSpawn,
    ReadyToSpawn,
    ClientReadyToSpawn,
}

impl PacketRead for RespawnState {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        match u8::read(reader)? {
            0 => Ok(Self::SearchingForSpawn),
            1 => Ok(Self::ReadyToSpawn),
            2 => Ok(Self::ClientReadyToSpawn),
            state => Err(Error::new(
                ErrorKind::InvalidData,
                format!("invalid Bedrock respawn state {state}"),
            )),
        }
    }
}

impl PacketWrite for RespawnState {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as u8).write(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serial::PacketRead;

    #[test]
    fn respawn_packet_roundtrip() {
        let packet = SRespawn {
            position: Vector3::new(1.5, 64.0, -2.25),
            state: RespawnState::ReadyToSpawn,
            player_runtime_id: VarULong(42),
        };
        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();

        let decoded = SRespawn::read(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded.position, packet.position);
        assert_eq!(decoded.state, packet.state);
        assert_eq!(decoded.player_runtime_id.0, packet.player_runtime_id.0);
    }
}

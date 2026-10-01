use pumpkin_data::packet::clientbound::play::RESPAWN;
use pumpkin_macros::java_packet;

use crate::{
    ClientPacket, ServerPacket,
    java::client::play::player_spawn_data::PlayerSpawnData,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

#[derive(Clone, Debug, PartialEq, Eq)]
#[java_packet(RESPAWN)]
pub struct CRespawn {
    pub player_spawn_info: PlayerSpawnData,
    pub data_kept: u8,
}

impl CRespawn {
    pub const KEEP_NOTHING: u8 = 0;
    pub const KEEP_ATTRIBUTES: u8 = 0b01;
    pub const KEEP_ATTRIBUTE_MODIFIERS: u8 = Self::KEEP_ATTRIBUTES;
    pub const KEEP_ENTITY_DATA: u8 = 0b10;
    pub const KEEP_ALL_DATA: u8 = Self::KEEP_ATTRIBUTES | Self::KEEP_ENTITY_DATA;

    #[must_use]
    pub const fn new(player_spawn_info: PlayerSpawnData, data_kept: u8) -> Self {
        Self {
            player_spawn_info,
            data_kept,
        }
    }
}

impl ClientPacket for CRespawn {
    fn write_packet_data(&self, mut write: impl std::io::Write) -> Result<(), WritingError> {
        self.player_spawn_info.write_packet_data(&mut write)?;
        write.write_u8(self.data_kept)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CRespawn {
    fn read(read: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let player_spawn_info = PlayerSpawnData::read(read)?;
        let data_kept = read.get_u8()?;
        Ok(Self::new(player_spawn_info, data_kept))
    }
}

use pumpkin_data::packet::clientbound::PLAY_RESPAWN;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ClientPacket,
    java::client::play::player_spawn_data::PlayerSpawnData,
    ser::{NetworkWriteExt, WritingError},
};

#[java_packet(PLAY_RESPAWN)]
pub struct CRespawn {
    pub player_spawn_info: PlayerSpawnData,
    pub data_kept: u8,
}

impl CRespawn {
    pub const KEEP_ATTRIBUTE_MODIFIERS: u8 = 1;
    pub const KEEP_ENTITY_DATA: u8 = 2;
    pub const KEEP_ALL_DATA: u8 = 3;

    #[must_use]
    pub const fn new(player_spawn_info: PlayerSpawnData, data_kept: u8) -> Self {
        Self {
            player_spawn_info,
            data_kept,
        }
    }
}

impl ClientPacket for CRespawn {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        self.player_spawn_info
            .write_packet_data(&mut write, version)?;
        write.write_i8(self.data_kept as i8)?;
        Ok(())
    }
}

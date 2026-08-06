use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_CHANGE_GAME_MODE;
use pumpkin_macros::java_packet;
use pumpkin_util::GameMode;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_CHANGE_GAME_MODE)]
pub struct SChangeGameMode {
    pub game_mode: GameMode,
}

impl<'a> ServerPacket<'a> for SChangeGameMode {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            game_mode: GameMode::try_from(bytebuf.get_u8()? as i8)
                .map_err(|()| crate::ser::ReadingError::Message("Invalid game mode".into()))?,
        })
    }
}

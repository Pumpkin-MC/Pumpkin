use pumpkin_data::packet::clientbound::PLAY_GAME_EVENT;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Updates the game state or triggers specific environmental changes.
///
/// This packet is the primary way the server communicates global or
/// context-specific transitions, such as changing the weather,
/// altering the player's gamemode, or displaying the credits.
#[java_packet(PLAY_GAME_EVENT)]
pub struct CGameEvent {
    /// The ID of the event type.
    pub event: u8,
    /// A value associated with the event (usage depends on the event ID).
    pub value: f32,
}

/// You need to implement all the random stuff somewhere, right?
impl CGameEvent {
    #[must_use]
    pub const fn new(event: GameEvent, value: f32) -> Self {
        Self {
            event: event as u8,
            value,
        }
    }
}

pub enum GameEvent {
    NoRespawnBlockAvailable,
    BeginRaining,
    EndRaining,
    ChangeGameMode,
    WinGame,
    DemoEvent,
    ArrowHitPlayer,
    RainLevelChange,
    ThunderLevelChange,
    PlayPufferfishStringSound,
    PlayElderGuardianMobAppearance,
    EnabledRespawnScreen,
    LimitedCrafting,
    StartWaitingChunks,
}

impl ClientPacket for CGameEvent {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_u8(self.event)?;
        write.write_f32_be(self.value)?;
        Ok(())
    }
}

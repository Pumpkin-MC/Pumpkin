use pumpkin_data::packet::clientbound::PLAY_GAME_EVENT;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

/// Updates the game state or triggers specific environmental changes.
///
/// This packet is the primary way the server communicates global or
/// context-specific transitions, such as changing the weather,
/// altering the player's gamemode, or displaying the credits.
#[derive(Serialize, Deserialize)]
#[java_packet(PLAY_GAME_EVENT)]
pub struct CGameEvent {
    /// The ID of the event type.
    pub event: u8,
    /// A value associated with the event (usage depends on the event ID).
    pub value: f32,
}

/// You need to implement all the random stuff somewhere, right?
impl CGameEvent {
    pub fn new(event: GameEvent, value: f32) -> Self {
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

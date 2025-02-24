use pumpkin_data::packet::clientbound::PLAY_GAME_EVENT;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_GAME_EVENT)]
pub struct CGameEvent {
    event: u8,
    value: f32,
}

/// Somewhere you need to implement all the random stuff right?
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

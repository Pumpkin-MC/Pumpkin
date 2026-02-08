use pumpkin_data::packet::serverbound::PLAY_LOCK_DIFFICULTY;
use pumpkin_macros::java_packet;

/// Sent when the player toggles the difficulty lock in the settings.
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_LOCK_DIFFICULTY)]
pub struct SLockDifficulty {
    pub locked: bool,
}

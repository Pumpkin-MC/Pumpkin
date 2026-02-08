use pumpkin_data::packet::serverbound::PLAY_CHANGE_DIFFICULTY;
use pumpkin_macros::java_packet;

/// Sent when the player changes the server difficulty from the settings menu.
///
/// 0 = Peaceful, 1 = Easy, 2 = Normal, 3 = Hard.
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_CHANGE_DIFFICULTY)]
pub struct SChangeDifficulty {
    pub difficulty: u8,
}

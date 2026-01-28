use pumpkin_data::packet::clientbound::PLAY_CHANGE_DIFFICULTY;
use pumpkin_macros::java_packet;
use serde::Serialize;

/// Notifies the client of a change in the world's difficulty level or lock status.
///
/// This updates the client's internal state, which affects certain UI elements
/// and client-side behavior (though actual game logic like mob damage is
/// primarily handled by the server).
#[derive(Serialize)]
#[java_packet(PLAY_CHANGE_DIFFICULTY)]
pub struct CChangeDifficulty {
    /// The current difficulty level of the world.
    ///
    /// * **0**: Peaceful
    /// * **1**: Easy
    /// * **2**: Normal
    /// * **3**: Hard
    pub difficulty: u8,
    /// Whether the difficulty is locked. If true, the client's difficulty
    /// toggle in the options menu will be disabled.
    pub locked: bool,
}

impl CChangeDifficulty {
    #[must_use]
    pub fn new(difficulty: u8, locked: bool) -> Self {
        Self { difficulty, locked }
    }
}

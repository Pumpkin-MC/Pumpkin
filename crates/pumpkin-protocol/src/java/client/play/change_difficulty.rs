use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_data::packet::clientbound::PLAY_CHANGE_DIFFICULTY;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

/// Notifies the client of a change in the world's difficulty level or lock status.
///
/// This updates the client's internal state, which affects certain UI elements
/// and client-side behavior (though actual game logic like mob damage is
/// primarily handled by the server).
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
    pub const fn new(difficulty: u8, locked: bool) -> Self {
        Self { difficulty, locked }
    }
}

impl ClientPacket for CChangeDifficulty {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_u8(self.difficulty)?;
        write.write_bool(self.locked)?;
        Ok(())
    }
}

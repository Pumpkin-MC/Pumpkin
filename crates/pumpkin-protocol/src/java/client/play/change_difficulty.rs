use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::CHANGE_DIFFICULTY;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

/// Notifies the client of a change in the world's difficulty level or lock status.
///
/// This updates the client's internal state, which affects certain UI elements
/// and client-side behavior (though actual game logic like mob damage is
/// primarily handled by the server).
#[java_packet(CHANGE_DIFFICULTY)]
pub struct CChangeDifficulty {
    /// The current difficulty level of the world.
    ///
    /// * **0**: Peaceful
    /// * **1**: Easy
    /// * **2**: Normal
    /// * **3**: Hard
    pub difficulty: u8,
    /// Whether the difficulty is locked (Added in 1.14). If true, the client's difficulty
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
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        // Difficulty enum serialized as VarInt in 1.21.6+, and unsigned byte before
        if *version >= JavaMinecraftVersion::V_1_21_6 {
            write.write_var_int(&VarInt(i32::from(self.difficulty)))?;
        } else {
            write.write_u8(self.difficulty)?;
        }
        // Added in 1.14: locked boolean
        if *version >= JavaMinecraftVersion::V_1_14 {
            write.write_bool(self.locked)?;
        }
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CChangeDifficulty {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let difficulty = if *version >= JavaMinecraftVersion::V_1_21_6 {
            bytebuf.get_var_int()?.0 as u8
        } else {
            bytebuf.get_u8()?
        };
        let locked = if *version >= JavaMinecraftVersion::V_1_14 {
            bytebuf.get_bool()?
        } else {
            false
        };
        Ok(Self { difficulty, locked })
    }
}

use pumpkin_data::packet::serverbound::play::CHANGE_DIFFICULTY;
use pumpkin_macros::java_packet;
use pumpkin_util::difficulty::Difficulty;

use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

#[java_packet(CHANGE_DIFFICULTY)]
pub struct SChangeDifficulty {
    pub difficulty: Difficulty,
}

impl SChangeDifficulty {
    #[must_use]
    pub const fn new(difficulty: Difficulty) -> Self {
        Self { difficulty }
    }
}

impl<'a> ServerPacket<'a> for SChangeDifficulty {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let id = bytebuf.get_var_int()?.0 as u8;
        let difficulty = match id {
            0 => Difficulty::Peaceful,
            1 => Difficulty::Easy,
            2 => Difficulty::Normal,
            3 => Difficulty::Hard,
            _ => {
                return Err(ReadingError::Message(format!(
                    "Invalid difficulty id: {id}"
                )));
            }
        };
        Ok(Self { difficulty })
    }
}

impl ClientPacket for SChangeDifficulty {
    fn write_packet_data(&self, mut write: impl std::io::Write) -> Result<(), WritingError> {
        write.write_var_int(&VarInt(self.difficulty as i32))?;
        Ok(())
    }
}

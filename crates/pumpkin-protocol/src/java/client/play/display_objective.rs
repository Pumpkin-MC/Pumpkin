use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::{
    packet::clientbound::play::SET_DISPLAY_OBJECTIVE, scoreboard::ScoreboardDisplaySlot,
};
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

/// Instructs the client to display a specific scoreboard objective in a given slot.
///
/// This packet is the final step in showing a scoreboard to a player. After
/// an objective is created and populated with scores, this packet "maps"
/// that objective to a visual location like the sidebar or the player list.
#[derive(Debug, PartialEq, Eq, Clone)]
#[java_packet(SET_DISPLAY_OBJECTIVE)]
pub struct CDisplayObjective {
    /// The display slot/position for the objective.
    pub position: VarInt,
    /// The unique internal name of the objective to be displayed.
    /// To hide an objective in a specific slot, send an empty string.
    pub score_name: String,
}

impl CDisplayObjective {
    #[must_use]
    pub const fn new(position: ScoreboardDisplaySlot, score_name: String) -> Self {
        Self {
            position: VarInt(position as i32),
            score_name,
        }
    }
}

impl ClientPacket for CDisplayObjective {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_20_2 {
            write.write_var_int(&self.position)?;
        } else {
            write.write_i8(self.position.0 as i8)?;
        }

        if *version >= JavaMinecraftVersion::V_1_18 {
            write.write_string(&self.score_name)?;
        } else {
            write.write_string_bounded(&self.score_name, 16)?;
        }
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CDisplayObjective {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let position = if *version >= JavaMinecraftVersion::V_1_20_2 {
            bytebuf.get_var_int()?
        } else {
            VarInt(i32::from(bytebuf.get_i8()?))
        };

        let score_name = if *version >= JavaMinecraftVersion::V_1_18 {
            bytebuf.get_str()?.into()
        } else {
            bytebuf.get_str_bounded(16)?.into()
        };

        Ok(Self {
            position,
            score_name,
        })
    }
}

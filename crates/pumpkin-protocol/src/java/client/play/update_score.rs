use pumpkin_data::packet::clientbound::PLAY_SET_SCORE;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::java_packet;

use crate::{
    ClientPacket, NumberFormat, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

/// Sent by the server to create or update a score for an entity on a specific objective.
///
/// This packet is the primary way to manage scoreboard data. In the latest protocol,
/// it also supports optional custom formatting for how the numeric score is displayed.
#[java_packet(PLAY_SET_SCORE)]
pub struct CUpdateScore {
    /// The name of the entity whose score is being updated (e.g., a player's username
    /// or a non-player entry like "Kills").
    pub entity_name: String,
    /// The internal name of the objective this score belongs to.
    pub objective_name: String,
    /// The actual integer value of the score.
    pub value: VarInt,
    /// An optional custom name for the entity to be displayed in the scoreboard.
    /// If `None`, the `entity_name` is used by default.
    pub display_name: Option<TextComponent>,
    /// Optional formatting for the number (e.g., blank, fixed text, or styled).
    /// This allows for scores to appear as something other than raw numbers.
    pub number_format: Option<NumberFormat>,
}

impl CUpdateScore {
    #[must_use]
    pub const fn new(
        entity_name: String,
        objective_name: String,
        value: VarInt,
        display_name: Option<TextComponent>,
        number_format: Option<NumberFormat>,
    ) -> Self {
        Self {
            entity_name,
            objective_name,
            value,
            display_name,
            number_format,
        }
    }

    #[must_use]
    pub const fn new_remove(entity_name: String, objective_name: String) -> Self {
        Self {
            entity_name,
            objective_name,
            value: VarInt(0),
            display_name: None,
            number_format: None,
        }
    }
}

impl ClientPacket for CUpdateScore {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &pumpkin_util::version::JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_string(&self.entity_name)?;
        write.write_string(&self.objective_name)?;
        write.write_var_int(&self.value)?;
        write.write_option(&self.display_name, |w, t| w.write_slice(&t.encode()))?;
        write.write_option(&self.number_format, |w, n| n.write(w))
    }
}

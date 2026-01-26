use pumpkin_data::packet::clientbound::PLAY_SET_SCORE;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::{NumberFormat, VarInt};

/// Sent by the server to create or update a score for an entity on a specific objective.
///
/// This packet is the primary way to manage scoreboard data. In the latest protocol,
/// it also supports optional custom formatting for how the numeric score is displayed.
#[derive(Serialize)]
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
    pub fn new(
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
}

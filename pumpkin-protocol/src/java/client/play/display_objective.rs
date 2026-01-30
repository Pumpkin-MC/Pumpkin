use pumpkin_data::{
    packet::clientbound::PLAY_SET_DISPLAY_OBJECTIVE, scoreboard::ScoreboardDisplaySlot,
};
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Instructs the client to display a specific scoreboard objective in a given slot.
///
/// This packet is the final step in showing a scoreboard to a player. After
/// an objective is created and populated with scores, this packet "maps"
/// that objective to a visual location like the sidebar or the player list.
#[derive(Serialize)]
#[java_packet(PLAY_SET_DISPLAY_OBJECTIVE)]
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

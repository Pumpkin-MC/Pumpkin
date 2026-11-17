// Last verified for v2169

use crate::{codec::var_int::VarInt, serial::PacketWrite};
use pumpkin_macros::packet;

/// Configures and displays a scoreboard objective in a specific UI slot.
#[derive(PacketWrite)]
#[packet(107)]
pub struct CSetDisplayObjective {
    /// Slot location where the objective is presented (`sidebar`, `list`, or `belowname`).
    pub display_slot_name: String,
    /// Internal unique identifier for the scoreboard objective.
    pub objective_name: String,
    /// Visible title text displayed atop the scoreboard.
    pub objective_display_name: String,
    /// Objective criteria string (e.g. `dummy`).
    pub criteria_name: String,
    /// Ordering direction for scores (0: ascending, 1: descending).
    pub sort_order: VarInt,
}

use pumpkin_data::packet::clientbound::PLAY_SET_SIMULATION_DISTANCE;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Tells the client the server's simulation distance.
///
/// Entities and block ticks outside this radius are not processed.
#[derive(Serialize)]
#[java_packet(PLAY_SET_SIMULATION_DISTANCE)]
pub struct CSetSimulationDistance {
    pub distance: VarInt,
}

impl CSetSimulationDistance {
    #[must_use]
    pub const fn new(distance: VarInt) -> Self {
        Self { distance }
    }
}

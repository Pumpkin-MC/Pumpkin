use pumpkin_data::packet::clientbound::PLAY_RESET_SCORE;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_RESET_SCORE)]
pub struct CResetScore {
    pub entity_name: String,
    pub objective_name: Option<String>,
}

impl CResetScore {
    #[must_use]
    pub fn new(entity_name: String, objective_name: Option<String>) -> Self {
        Self {
            entity_name,
            objective_name,
        }
    }
}

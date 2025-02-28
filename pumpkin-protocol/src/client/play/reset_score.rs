use pumpkin_data::packet::clientbound::PLAY_RESET_SCORE;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_RESET_SCORE)]
pub struct CResetScore {
    entity_name: String,
    objective_name: Option<String>,
}

impl CResetScore {
    pub fn new(entity_name: String, objective_name: Option<String>) -> Self {
        Self {
            entity_name,
            objective_name,
        }
    }
}

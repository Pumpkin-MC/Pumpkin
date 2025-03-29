use std::borrow::Cow;

use pumpkin_data::packet::clientbound::PLAY_RESET_SCORE;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_RESET_SCORE)]
pub struct CResetScore<'a> {
    entity_name: Cow<'a, str>,
    objective_name: Option<String>,
}

impl<'a> CResetScore<'a> {
    pub fn new(entity_name: Cow<'a, str>, objective_name: Option<String>) -> Self {
        Self {
            entity_name,
            objective_name,
        }
    }
}

use pumpkin_data::packet::serverbound::CONFIG_FINISH_CONFIGURATION;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(CONFIG_FINISH_CONFIGURATION)]
pub struct SAcknowledgeFinishConfig;

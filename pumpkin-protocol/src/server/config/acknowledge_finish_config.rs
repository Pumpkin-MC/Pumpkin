use pumpkin_data::packet::serverbound::CONFIG_FINISH_CONFIGURATION;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(CONFIG_FINISH_CONFIGURATION)]
pub struct SAcknowledgeFinishConfig;

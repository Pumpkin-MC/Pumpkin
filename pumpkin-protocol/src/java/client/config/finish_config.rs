use pumpkin_data::packet::clientbound::CONFIG_FINISH_CONFIGURATION;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(CONFIG_FINISH_CONFIGURATION)]
pub struct CFinishConfig;

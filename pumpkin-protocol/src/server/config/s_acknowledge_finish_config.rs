use pumpkin_data::packets::serverbound::CONFIG_FINISH_CONFIGURATION;
use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet(CONFIG_FINISH_CONFIGURATION)]
pub struct SAcknowledgeFinishConfig {}

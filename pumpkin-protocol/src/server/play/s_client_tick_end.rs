use pumpkin_data::packets::serverbound::PLAY_CLIENT_TICK_END;
use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet(PLAY_CLIENT_TICK_END)]
pub struct SClientTickEnd {}

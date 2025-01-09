use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("play:player_loaded")]
pub struct SPlayerLoaded;

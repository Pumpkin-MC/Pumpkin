use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_STATUS_ONLY;
use pumpkin_macros::server_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet(PLAY_MOVE_PLAYER_STATUS_ONLY)]
pub struct SSetPlayerGround {
    pub on_ground: bool,
}

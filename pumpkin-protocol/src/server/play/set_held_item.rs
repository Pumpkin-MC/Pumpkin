use pumpkin_data::packet::serverbound::PLAY_SET_CARRIED_ITEM;
use pumpkin_macros::server_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet(PLAY_SET_CARRIED_ITEM)]
pub struct SSetHeldItem {
    pub slot: i16,
}

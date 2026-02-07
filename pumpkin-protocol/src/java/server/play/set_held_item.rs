use pumpkin_data::packet::serverbound::PLAY_SET_CARRIED_ITEM;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[java_packet(PLAY_SET_CARRIED_ITEM)]
pub struct SSetHeldItem {
    pub slot: i16,
}

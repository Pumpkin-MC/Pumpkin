use crate::VarInt;
use pumpkin_data::packet::serverbound::PLAY_CONTAINER_BUTTON_CLICK;
use pumpkin_macros::java_packet;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[java_packet(PLAY_CONTAINER_BUTTON_CLICK)]
pub struct SContainerButtonClick {
    pub window_id: VarInt,
    pub button_id: VarInt,
}

use pumpkin_data::packet::serverbound::PLAY_CUSTOM_PAYLOAD;
use pumpkin_macros::java_packet;

// TODO
#[java_packet(PLAY_CUSTOM_PAYLOAD)]
pub struct SCustomPayload;

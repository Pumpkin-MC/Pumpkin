use pumpkin_data::packet::serverbound::PLAY_CLIENT_TICK_END;
use pumpkin_macros::java_packet;

#[java_packet(PLAY_CLIENT_TICK_END)]
pub struct SClientTickEnd;

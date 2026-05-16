use crate::{codec::var_int::VarInt, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(88)]
pub struct CSetTitle {
    pub action_type: VarInt,
    pub text: String,
    pub fade_in_duration: VarInt,
    pub remain_duration: VarInt,
    pub fade_out_duration: VarInt,
    pub xuid: String,
    pub platform_online_id: String,
    pub filtered_message: String,
}

impl CSetTitle {
    #[must_use]
    pub const fn new(
        action_type: i32,
        text: String,
        fade_in_duration: i32,
        remain_duration: i32,
        fade_out_duration: i32,
    ) -> Self {
        Self {
            action_type: VarInt(action_type),
            text,
            fade_in_duration: VarInt(fade_in_duration),
            remain_duration: VarInt(remain_duration),
            fade_out_duration: VarInt(fade_out_duration),
            xuid: String::new(),
            platform_online_id: String::new(),
            filtered_message: String::new(),
        }
    }
}

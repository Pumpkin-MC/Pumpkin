// Last verified for v2169

use crate::{codec::var_int::VarInt, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(88)]
pub struct CSetTitle {
    pub title_type: TitleType,
    pub title_text: String,
    pub fade_in_time: VarInt,
    pub stay_time: VarInt,
    pub fade_out_time: VarInt,
    pub xuid: String,
    pub platform_online_id: String,
    pub filtered_title_message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PacketWrite)]
#[repr(i32)]
#[serial(varint)]
pub enum TitleType {
    Clear,
    Reset,
    Title,
    Subtitle,
    Actionbar,
    Times,
    TitleTextObject,
    SubtitleTextObject,
    ActionbarTextObject,
}

impl CSetTitle {
    #[must_use]
    pub const fn new(
        title_type: TitleType,
        title_text: String,
        fade_in_time: i32,
        stay_time: i32,
        fade_out_time: i32,
    ) -> Self {
        Self {
            title_type,
            title_text,
            fade_in_time: VarInt(fade_in_time),
            stay_time: VarInt(stay_time),
            fade_out_time: VarInt(fade_out_time),
            xuid: String::new(),
            platform_online_id: String::new(),
            filtered_title_message: String::new(),
        }
    }
}

use pumpkin_data::packet::clientbound::PLAY_AWARD_STATS;
use pumpkin_macros::java_packet;

use crate::codec::var_int::VarInt;

#[derive(serde::Serialize)]
#[java_packet(PLAY_AWARD_STATS)]
pub struct CAwardStats<'a> {
    pub stats: &'a [Statistic],
}

#[derive(serde::Serialize)]
pub struct Statistic {
    pub category_id: VarInt,
    pub statistic_id: VarInt,
    pub value: VarInt,
}

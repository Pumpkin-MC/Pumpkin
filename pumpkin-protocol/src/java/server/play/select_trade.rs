use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_SELECT_TRADE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(PLAY_SELECT_TRADE)]
pub struct SSelectMerchantTrade {
    pub selected_slot: VarInt,
}

impl ServerPacket for SSelectMerchantTrade {
    fn read(mut read: impl Read, _version: &MinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            selected_slot: read.get_var_int()?,
        })
    }
}

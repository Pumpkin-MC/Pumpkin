// Last verified for v2169

use std::io::{Error, Read};

use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketRead};

#[derive(PacketRead)]
#[packet(312)]
pub struct SLoadingScreen {
    loading_screen_packet_type: LoadingScreenPacketType,
    _loading_screen_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum LoadingScreenPacketType {
    StartLoadingScreen = 0,
    EndLoadingScreen = 1,
}

impl PacketRead for LoadingScreenPacketType {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        match VarInt::read(reader)?.0 {
            0 => Ok(Self::StartLoadingScreen),
            1 => Ok(Self::EndLoadingScreen),
            val => Err(Error::other(format!(
                "Invalid LoadingScreenPacketType: {val}"
            ))),
        }
    }
}

impl SLoadingScreen {
    #[must_use]
    pub fn is_loading_done(&self) -> bool {
        self.loading_screen_packet_type == LoadingScreenPacketType::EndLoadingScreen
    }
}

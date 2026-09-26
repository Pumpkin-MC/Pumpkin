// Last verified for v2169

use pumpkin_macros::packet;

use crate::serial::PacketRead;

#[derive(PacketRead)]
#[packet(312)]
pub struct SLoadingScreen {
    loading_screen_packet_type: LoadingScreenPacketType,
    _loading_screen_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PacketRead)]
#[repr(i32)]
#[serial(varint)]
pub enum LoadingScreenPacketType {
    StartLoadingScreen = 0,
    EndLoadingScreen = 1,
}

impl SLoadingScreen {
    #[must_use]
    pub fn is_loading_done(&self) -> bool {
        self.loading_screen_packet_type == LoadingScreenPacketType::EndLoadingScreen
    }
}

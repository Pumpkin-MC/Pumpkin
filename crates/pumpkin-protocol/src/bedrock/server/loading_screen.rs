// Last verified for v2169

use pumpkin_macros::packet;

use crate::serial::PacketRead;

/// Sent by the client to update the server on client-side loading screen transitions.
#[derive(PacketRead)]
#[packet(312)]
pub struct SLoadingScreen {
    loading_screen_packet_type: LoadingScreenPacketType,
    _loading_screen_id: Option<u32>,
}

/// Type of loading screen transition event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PacketRead)]
#[repr(i32)]
#[serial(varint)]
pub enum LoadingScreenPacketType {
    /// Client has started displaying a loading screen.
    StartLoadingScreen = 0,
    /// Client has dismissed the loading screen and resumed rendering.
    EndLoadingScreen = 1,
}

impl SLoadingScreen {
    #[must_use]
    pub fn is_loading_done(&self) -> bool {
        self.loading_screen_packet_type == LoadingScreenPacketType::EndLoadingScreen
    }
}

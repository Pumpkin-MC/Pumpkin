use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketRead};

#[derive(PacketRead)]
#[packet(312)]
pub struct SLoadingScreen {
    // https://mojang.github.io/bedrock-protocol-docs/html/ServerboundLoadingScreenPacket.html
    // Loading Screen Packet Type
    // 0: Inavil, 1: Start, 2: End
    status: VarInt,
    _id: Option<u32>,
}

impl SLoadingScreen {
    #[must_use]
    pub fn is_loading_done(&self) -> bool {
        self.status.0 == 2
    }
}

use crate::serial::PacketRead;
use pumpkin_macros::packet;

#[derive(PacketRead)]
#[packet(8)]
pub struct SResourcePackResponse {
    pub response: u8,
    pub download_size: u16,
}

impl SResourcePackResponse {
    pub const STATUS_REFUSED: u8 = 1;
    pub const STATUS_SEND_PACKS: u8 = 2;
    pub const STATUS_HAVE_ALL_PACKS: u8 = 3;
    pub const STATUS_COMPLETED: u8 = 4;
}

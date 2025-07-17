use crate::serial::PacketRead;
use pumpkin_macros::packet;

#[derive(PacketRead)]
#[packet(8)]
pub struct SRessourcePackResponse {
    pub response: u8,
    pub download_size: u16,
}

use crate::serial::PacketWrite;
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(186)]
pub struct CToastRequest {
    pub title: String,
    pub content: String,
}

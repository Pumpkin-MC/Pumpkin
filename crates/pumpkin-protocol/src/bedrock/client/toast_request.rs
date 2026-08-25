use crate::serial::PacketWrite;
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(186)]
pub struct CToastRequest<'a> {
    pub title: &'a str,
    pub content: &'a str,
}

use crate::serial::PacketWrite;
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(0x03)]
pub struct CHandshake {
    jwt_data: String,
}

impl CHandshake {
    #[must_use]
    pub const fn new(jwt_data: String) -> Self {
        Self { jwt_data }
    }
}
